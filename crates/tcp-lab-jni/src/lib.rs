use jni::JNIEnv;
use jni::objects::{JClass, JObject, JString, JValue, JByteArray};
use jni::sys::{jint, jlong, jbyte, jbyteArray};
use std::cell::RefCell;
use tcp_lab_core::{SystemContext, TransportProtocol, Packet, TcpHeader};
use tracing::error;
use std::sync::Arc;

// ==========================================
// TLS Context Management
// ==========================================

// Use usize to store the raw pointer to avoid lifetime issues with thread_local!
thread_local! {
    static CURRENT_CONTEXT_PTR: RefCell<usize> = RefCell::new(0);
}

fn with_context<F, R>(ctx: &mut dyn SystemContext, f: F) -> R
where
    F: FnOnce() -> R,
{
    let ptr = ctx as *mut dyn SystemContext;
    let static_ptr: *mut (dyn SystemContext + 'static) = unsafe { std::mem::transmute(ptr) };
    
    CURRENT_CONTEXT.with(|c| {
        *c.borrow_mut() = Some(static_ptr);
    });

    let result = f();

    CURRENT_CONTEXT.with(|c| {
        *c.borrow_mut() = None;
    });
    result
}

thread_local! {
    static CURRENT_CONTEXT: RefCell<Option<*mut (dyn SystemContext + 'static)>> = RefCell::new(None);
}

fn use_context<F>(f: F)
where
    F: FnOnce(&mut dyn SystemContext),
{
    CURRENT_CONTEXT.with(|c| {
        if let Some(ptr) = *c.borrow() {
            let ctx = unsafe { &mut *ptr };
            f(ctx);
        } else {
            error!("Java called native method without active SystemContext!");
        }
    });
}

// ==========================================
// Native Methods Implementation
// ==========================================

#[no_mangle]
pub extern "system" fn Java_com_ouc_tcp_sdk_NativeBridge_sendPacket(
    env: JNIEnv,
    _class: JClass,
    seq: jlong,
    ack: jlong,
    flags: jbyte,
    window: jint,
    checksum: jint,
    payload: jbyteArray,
) {
    let payload_vec = match env.convert_byte_array(unsafe { JByteArray::from_raw(payload) }) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to convert byte array: {:?}", e);
            return;
        }
    };

    use_context(|ctx| {
        let header = TcpHeader {
            seq_num: seq as u32,
            ack_num: ack as u32,
            flags: flags as u8,
            window_size: window as u16,
            checksum: checksum as u16,
            ..Default::default()
        };
        let packet = Packet::new(header, payload_vec);
        ctx.send_packet(packet);
    });
}

#[no_mangle]
pub extern "system" fn Java_com_ouc_tcp_sdk_NativeBridge_startTimer(
    _env: JNIEnv,
    _class: JClass,
    delay_ms: jlong,
    timer_id: jint,
) {
    use_context(|ctx| {
        ctx.start_timer(delay_ms as u64, timer_id as u32);
    });
}

#[no_mangle]
pub extern "system" fn Java_com_ouc_tcp_sdk_NativeBridge_cancelTimer(
    _env: JNIEnv,
    _class: JClass,
    timer_id: jint,
) {
    use_context(|ctx| {
        ctx.cancel_timer(timer_id as u32);
    });
}

#[no_mangle]
pub extern "system" fn Java_com_ouc_tcp_sdk_NativeBridge_deliverData(
    env: JNIEnv,
    _class: JClass,
    data: jbyteArray,
) {
    let data_vec = match env.convert_byte_array(unsafe { JByteArray::from_raw(data) }) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to convert byte array: {:?}", e);
            return;
        }
    };

    use_context(|ctx| {
        ctx.deliver_data(&data_vec);
    });
}

#[no_mangle]
pub extern "system" fn Java_com_ouc_tcp_sdk_NativeBridge_log(
    mut env: JNIEnv,
    _class: JClass,
    msg: JString,
) {
    let msg_str: String = match env.get_string(&msg) {
        Ok(s) => s.into(),
        Err(_) => "Invalid UTF-8 string in log".into(),
    };

    use_context(|ctx| {
        ctx.log(&msg_str);
    });
}

#[no_mangle]
pub extern "system" fn Java_com_ouc_tcp_sdk_NativeBridge_now(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    let mut time = 0;
    use_context(|ctx| {
        time = ctx.now() as i64;
    });
    time
}

// ==========================================
// Native Registration
// ==========================================

pub fn register_native_methods(env: &mut JNIEnv) -> jni::errors::Result<()> {
    let class = env.find_class("com/ouc/tcp/sdk/NativeBridge")?;
    let methods = [
        jni::NativeMethod {
            name: "sendPacket".into(),
            sig: "(JJBII[B)V".into(),
            fn_ptr: Java_com_ouc_tcp_sdk_NativeBridge_sendPacket as *mut _,
        },
        jni::NativeMethod {
            name: "startTimer".into(),
            sig: "(JI)V".into(),
            fn_ptr: Java_com_ouc_tcp_sdk_NativeBridge_startTimer as *mut _,
        },
        jni::NativeMethod {
            name: "cancelTimer".into(),
            sig: "(I)V".into(),
            fn_ptr: Java_com_ouc_tcp_sdk_NativeBridge_cancelTimer as *mut _,
        },
        jni::NativeMethod {
            name: "deliverData".into(),
            sig: "([B)V".into(),
            fn_ptr: Java_com_ouc_tcp_sdk_NativeBridge_deliverData as *mut _,
        },
        jni::NativeMethod {
            name: "log".into(),
            sig: "(Ljava/lang/String;)V".into(),
            fn_ptr: Java_com_ouc_tcp_sdk_NativeBridge_log as *mut _,
        },
        jni::NativeMethod {
            name: "now".into(),
            sig: "()J".into(),
            fn_ptr: Java_com_ouc_tcp_sdk_NativeBridge_now as *mut _,
        },
    ];
    env.register_native_methods(class, &methods)
}

// ==========================================
// Rust Wrapper for Java Protocol
// ==========================================

pub struct JavaTransportProtocol {
    jvm: Arc<jni::JavaVM>,
    instance: Option<jni::objects::GlobalRef>,
    context_impl: Option<jni::objects::GlobalRef>,
}

impl JavaTransportProtocol {
    pub fn new(jvm: Arc<jni::JavaVM>, instance: jni::objects::GlobalRef) -> Self {
        let ctx_ref = {
            let mut env = jvm.attach_current_thread().expect("Failed to attach thread");
            let ctx_cls = env.find_class("com/ouc/tcp/sdk/SystemContextImpl").expect("Failed to find SystemContextImpl");
            let ctx_obj = env.new_object(ctx_cls, "()V", &[]).expect("Failed to create SystemContextImpl");
            env.new_global_ref(ctx_obj).expect("Failed to create global ref")
        };

        Self {
            jvm,
            instance: Some(instance),
            context_impl: Some(ctx_ref),
        }
    }

    fn call_java<F>(&mut self, ctx: &mut dyn SystemContext, op: F)
    where
        F: FnOnce(&mut JNIEnv, &JObject, &JObject) -> jni::errors::Result<()>,
    {
        let mut env = match self.jvm.attach_current_thread() {
            Ok(e) => e,
            Err(e) => {
                error!("Failed to attach JNI thread: {:?}", e);
                return;
            }
        };

        with_context(ctx, || {
            let obj = self.instance.as_ref().unwrap().as_obj();
            let ctx_obj = self.context_impl.as_ref().unwrap().as_obj();
            
            if let Err(e) = op(&mut env, &obj, &ctx_obj) {
                error!("Java exception or JNI error: {:?}", e);
                if env.exception_check().unwrap_or(false) {
                    env.exception_describe().unwrap_or(());
                    env.exception_clear().unwrap_or(());
                }
            }
        });
    }
}

impl Drop for JavaTransportProtocol {
    fn drop(&mut self) {
        // Attach current thread to JVM to safely drop GlobalRefs
        let _guard = self.jvm.attach_current_thread().ok();
        
        // Explicitly drop the GlobalRefs while we are attached
        self.instance = None;
        self.context_impl = None;
    }
}

impl TransportProtocol for JavaTransportProtocol {
    fn init(&mut self, ctx: &mut dyn SystemContext) {
        self.call_java(ctx, |env, obj, ctx_obj| {
            env.call_method(
                obj, 
                "init", 
                "(Lcom/ouc/tcp/sdk/SystemContext;)V", 
                &[JValue::Object(ctx_obj)]
            )?;
            Ok(())
        });
    }

    fn on_packet(&mut self, ctx: &mut dyn SystemContext, packet: Packet) {
        self.call_java(ctx, |env, obj, ctx_obj| {
            let header_cls = env.find_class("com/ouc/tcp/sdk/TcpHeader")?;
            let header_obj = env.new_object(header_cls, "()V", &[])?;
            
            env.call_method(&header_obj, "setSeqNum", "(J)V", &[JValue::Long(packet.header.seq_num as i64)])?;
            env.call_method(&header_obj, "setAckNum", "(J)V", &[JValue::Long(packet.header.ack_num as i64)])?;
            env.call_method(&header_obj, "setFlags", "(B)V", &[JValue::Byte(packet.header.flags as i8)])?;
            env.call_method(&header_obj, "setWindowSize", "(I)V", &[JValue::Int(packet.header.window_size as i32)])?;
            env.call_method(&header_obj, "setChecksum", "(I)V", &[JValue::Int(packet.header.checksum as i32)])?;
            
            let payload_arr = env.byte_array_from_slice(&packet.payload)?;
            
            let packet_cls = env.find_class("com/ouc/tcp/sdk/Packet")?;
            let packet_obj = env.new_object(
                packet_cls, 
                "(Lcom/ouc/tcp/sdk/TcpHeader;[B)V", 
                &[JValue::Object(&header_obj), JValue::Object(&payload_arr)]
            )?;

            env.call_method(
                obj,
                "onPacket",
                "(Lcom/ouc/tcp/sdk/SystemContext;Lcom/ouc/tcp/sdk/Packet;)V",
                &[JValue::Object(ctx_obj), JValue::Object(&packet_obj)]
            )?;
            Ok(())
        });
    }

    fn on_timer(&mut self, ctx: &mut dyn SystemContext, timer_id: u32) {
        self.call_java(ctx, |env, obj, ctx_obj| {
            env.call_method(
                obj,
                "onTimer",
                "(Lcom/ouc/tcp/sdk/SystemContext;I)V",
                &[JValue::Object(ctx_obj), JValue::Int(timer_id as i32)]
            )?;
            Ok(())
        });
    }

    fn send(&mut self, ctx: &mut dyn SystemContext, data: &[u8]) {
        self.call_java(ctx, |env, obj, ctx_obj| {
            let data_arr = env.byte_array_from_slice(data)?;
            env.call_method(
                obj,
                "onAppData",
                "(Lcom/ouc/tcp/sdk/SystemContext;[B)V",
                &[JValue::Object(ctx_obj), JValue::Object(&data_arr)]
            )?;
            Ok(())
        });
    }
}

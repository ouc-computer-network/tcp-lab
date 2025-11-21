use std::cell::RefCell;
use std::ptr;
use std::slice;

use tcp_lab_abstract::{Packet, SystemContext, TcpHeader};
use tracing::error;

// ==========================================
// TLS Context Management (same pattern as JNI)
// ==========================================

thread_local! {
    static CURRENT_CONTEXT: RefCell<Option<*mut (dyn SystemContext + 'static)>> =
        RefCell::new(None);
}

/// Ensure that the C ABI symbols remain linked/exported when the host application
/// only uses the TLS utilities from this crate.
#[inline(never)]
pub fn ensure_linked() {
    unsafe {
        ptr::read_volatile(
            &(tcp_lab_send_packet
                as unsafe extern "C" fn(u32, u32, u8, u16, u16, *const u8, usize)),
        );
        ptr::read_volatile(&(tcp_lab_start_timer as unsafe extern "C" fn(u64, i32)));
        ptr::read_volatile(&(tcp_lab_cancel_timer as unsafe extern "C" fn(i32)));
        ptr::read_volatile(&(tcp_lab_deliver_data as unsafe extern "C" fn(*const u8, usize)));
        ptr::read_volatile(&(tcp_lab_log as unsafe extern "C" fn(*const i8)));
        ptr::read_volatile(&(tcp_lab_now as unsafe extern "C" fn() -> u64));
        ptr::read_volatile(&(tcp_lab_record_metric as unsafe extern "C" fn(*const i8, f64)));
    }
}

/// Run `f` with the given SystemContext installed in TLS so that `tcp_lab_*`
/// functions can find it.
pub fn with_context<F, R>(ctx: &mut dyn SystemContext, f: F) -> R
where
    F: FnOnce() -> R,
{
    let ptr = ctx as *mut dyn SystemContext;
    // Extend lifetime to 'static for storage in TLS
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

fn use_context<F>(f: F)
where
    F: FnOnce(&mut dyn SystemContext),
{
    CURRENT_CONTEXT.with(|c| {
        if let Some(ptr) = *c.borrow() {
            let ctx = unsafe { &mut *ptr };
            f(ctx);
        } else {
            error!("tcp-lab-ffi: called without active SystemContext!");
        }
    });
}

// ==========================================
// C ABI functions used by C++ SDK (NativeBridge.hpp)
// ==========================================

#[unsafe(no_mangle)]
pub extern "C" fn tcp_lab_send_packet(
    seq: u32,
    ack: u32,
    flags: u8,
    window: u16,
    checksum: u16,
    payload: *const u8,
    payload_len: usize,
) {
    if payload.is_null() && payload_len > 0 {
        error!("tcp_lab_send_packet called with null payload pointer");
        return;
    }

    let data = if payload_len == 0 {
        Vec::new()
    } else {
        unsafe { slice::from_raw_parts(payload, payload_len) }.to_vec()
    };

    use_context(|ctx| {
        let header = TcpHeader {
            seq_num: seq,
            ack_num: ack,
            flags,
            window_size: window,
            checksum,
            ..Default::default()
        };
        let packet = Packet::new(header, data);
        ctx.send_packet(packet);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn tcp_lab_start_timer(delay_ms: u64, timer_id: i32) {
    use_context(|ctx| {
        ctx.start_timer(delay_ms, timer_id as u32);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn tcp_lab_cancel_timer(timer_id: i32) {
    use_context(|ctx| {
        ctx.cancel_timer(timer_id as u32);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn tcp_lab_deliver_data(data: *const u8, len: usize) {
    if data.is_null() {
        if len > 0 {
            error!("tcp_lab_deliver_data called with null data pointer");
        }
        use_context(|ctx| {
            ctx.deliver_data(&[]);
        });
        return;
    }

    let slice = unsafe { slice::from_raw_parts(data, len) };

    use_context(|ctx| {
        ctx.deliver_data(slice);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn tcp_lab_log(msg: *const i8) {
    if msg.is_null() {
        return;
    }
    unsafe {
        let cstr = std::ffi::CStr::from_ptr(msg);
        if let Ok(s) = cstr.to_str() {
            use_context(|ctx| {
                ctx.log(s);
            });
        } else {
            error!("tcp_lab_log received invalid UTF-8");
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn tcp_lab_now() -> u64 {
    let mut time = 0u64;
    use_context(|ctx| {
        time = ctx.now();
    });
    time
}

#[unsafe(no_mangle)]
pub extern "C" fn tcp_lab_record_metric(name: *const i8, value: f64) {
    if name.is_null() {
        return;
    }
    unsafe {
        let cstr = std::ffi::CStr::from_ptr(name);
        if let Ok(s) = cstr.to_str() {
            use_context(|ctx| {
                ctx.record_metric(s, value);
            });
        } else {
            error!("tcp_lab_record_metric received invalid UTF-8 name");
        }
    }
}

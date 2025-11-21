use std::ffi::c_void;
use std::path::Path;

use anyhow::Context;
use libloading::{Library, Symbol};
use tcp_lab_abstract::{Packet, SystemContext, TransportProtocol};
use tcp_lab_ffi::with_context;

/// C function types exported by a C++ protocol library.
///
/// The expected C++ signatures are:
/// ```cpp
/// extern "C" TransportProtocol* create_protocol();
/// extern "C" void destroy_protocol(TransportProtocol*);
/// extern "C" void protocol_init(TransportProtocol*);
/// extern "C" void protocol_on_app_data(TransportProtocol*, const uint8_t* data, size_t len);
/// extern "C" void protocol_on_packet(TransportProtocol*,
///                                  uint32_t seq, uint32_t ack, uint8_t flags,
///                                  uint16_t window, uint16_t checksum,
///                                  const uint8_t* payload, size_t len);
/// extern "C" void protocol_on_timer(TransportProtocol*, int timerId);
/// ```

type CreateFn = unsafe extern "C" fn() -> *mut c_void;
type DestroyFn = unsafe extern "C" fn(*mut c_void);
type InitFn = unsafe extern "C" fn(*mut c_void);
type OnAppDataFn = unsafe extern "C" fn(*mut c_void, *const u8, usize);
type OnPacketFn = unsafe extern "C" fn(*mut c_void, u32, u32, u8, u16, u16, *const u8, usize);
type OnTimerFn = unsafe extern "C" fn(*mut c_void, i32);

pub struct CppTransportProtocol {
    _lib: Library,
    instance: *mut c_void,
    destroy: DestroyFn,
    init_fn: InitFn,
    on_app_data_fn: OnAppDataFn,
    on_packet_fn: OnPacketFn,
    on_timer_fn: OnTimerFn,
}

unsafe impl Send for CppTransportProtocol {}
unsafe impl Sync for CppTransportProtocol {}

impl CppTransportProtocol {
    fn new(lib: Library) -> anyhow::Result<Self> {
        unsafe {
            let create: Symbol<CreateFn> = lib
                .get(b"create_protocol\0")
                .context("missing create_protocol")?;
            let destroy_sym: Symbol<DestroyFn> = lib
                .get(b"destroy_protocol\0")
                .context("missing destroy_protocol")?;
            let init_sym: Symbol<InitFn> = lib
                .get(b"protocol_init\0")
                .context("missing protocol_init")?;
            let on_app_data_sym: Symbol<OnAppDataFn> = lib
                .get(b"protocol_on_app_data\0")
                .context("missing protocol_on_app_data")?;
            let on_packet_sym: Symbol<OnPacketFn> = lib
                .get(b"protocol_on_packet\0")
                .context("missing protocol_on_packet")?;
            let on_timer_sym: Symbol<OnTimerFn> = lib
                .get(b"protocol_on_timer\0")
                .context("missing protocol_on_timer")?;

            let destroy = *destroy_sym;
            let init_fn = *init_sym;
            let on_app_data_fn = *on_app_data_sym;
            let on_packet_fn = *on_packet_sym;
            let on_timer_fn = *on_timer_sym;

            let instance = create();
            if instance.is_null() {
                anyhow::bail!("create_protocol returned null");
            }

            Ok(Self {
                _lib: lib,
                instance,
                destroy,
                init_fn,
                on_app_data_fn,
                on_packet_fn,
                on_timer_fn,
            })
        }
    }
}

impl Drop for CppTransportProtocol {
    fn drop(&mut self) {
        unsafe {
            (self.destroy)(self.instance);
        }
    }
}

impl TransportProtocol for CppTransportProtocol {
    fn init(&mut self, ctx: &mut dyn SystemContext) {
        unsafe {
            with_context(ctx, || {
                (self.init_fn)(self.instance);
            });
        }
    }

    fn on_packet(&mut self, ctx: &mut dyn SystemContext, packet: Packet) {
        unsafe {
            let header = packet.header;
            let payload = packet.payload;
            with_context(ctx, || {
                (self.on_packet_fn)(
                    self.instance,
                    header.seq_num,
                    header.ack_num,
                    header.flags,
                    header.window_size,
                    header.checksum,
                    payload.as_ptr(),
                    payload.len(),
                );
            });
        }
    }

    fn on_timer(&mut self, ctx: &mut dyn SystemContext, timer_id: u32) {
        unsafe {
            with_context(ctx, || {
                (self.on_timer_fn)(self.instance, timer_id as i32);
            });
        }
    }

    fn on_app_data(&mut self, ctx: &mut dyn SystemContext, data: &[u8]) {
        unsafe {
            with_context(ctx, || {
                (self.on_app_data_fn)(self.instance, data.as_ptr(), data.len());
            });
        }
    }
}

/// Load a C++ protocol library from the given path and wrap it as a Rust TransportProtocol.
pub fn load_protocol<P: AsRef<Path>>(path: P) -> anyhow::Result<Box<dyn TransportProtocol>> {
    let lib = unsafe { Library::new(path.as_ref()) }
        .with_context(|| format!("failed to load C++ protocol library {:?}", path.as_ref()))?;
    let cpp = CppTransportProtocol::new(lib)?;
    Ok(Box::new(cpp))
}

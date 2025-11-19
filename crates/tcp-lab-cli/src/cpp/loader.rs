use std::ffi::c_void;
use std::path::Path;

use anyhow::Context;
use libloading::{Library, Symbol};
use tcp_lab_core::{Packet, TransportProtocol, SystemContext};
use tcp_lab_ffi::with_context;

/// C function types exported by a C++ protocol library.
///
/// The expected C++ signatures (for a sender library) are roughly:
/// ```cpp
/// extern "C" TransportProtocol* create_sender();
/// extern "C" void destroy_sender(TransportProtocol*);
/// extern "C" void sender_init(TransportProtocol*);
/// extern "C" void sender_on_app_data(TransportProtocol*, const uint8_t* data, size_t len);
/// extern "C" void sender_on_packet(TransportProtocol*,
///                                  uint32_t seq, uint32_t ack, uint8_t flags,
///                                  uint16_t window, uint16_t checksum,
///                                  const uint8_t* payload, size_t len);
/// extern "C" void sender_on_timer(TransportProtocol*, int timerId);
/// ```

type CreateFn = unsafe extern "C" fn() -> *mut c_void;
type DestroyFn = unsafe extern "C" fn(*mut c_void);
type InitFn = unsafe extern "C" fn(*mut c_void);
type OnAppDataFn = unsafe extern "C" fn(*mut c_void, *const u8, usize);
type OnPacketFn =
    unsafe extern "C" fn(*mut c_void, u32, u32, u8, u16, u16, *const u8, usize);
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
            // Load symbols first, keeping the library borrowed only within this scope.
            let create: Symbol<CreateFn> =
                lib.get(b"create_sender\0").context("missing create_sender")?;
            let destroy_sym: Symbol<DestroyFn> =
                lib.get(b"destroy_sender\0").context("missing destroy_sender")?;
            let init_sym: Symbol<InitFn> =
                lib.get(b"sender_init\0").context("missing sender_init")?;
            let on_app_data_sym: Symbol<OnAppDataFn> =
                lib.get(b"sender_on_app_data\0")
                    .context("missing sender_on_app_data")?;
            let on_packet_sym: Symbol<OnPacketFn> =
                lib.get(b"sender_on_packet\0").context("missing sender_on_packet")?;
            let on_timer_sym: Symbol<OnTimerFn> =
                lib.get(b"sender_on_timer\0").context("missing sender_on_timer")?;

            let destroy = *destroy_sym;
            let init_fn = *init_sym;
            let on_app_data_fn = *on_app_data_sym;
            let on_packet_fn = *on_packet_sym;
            let on_timer_fn = *on_timer_sym;

            let instance = create();
            if instance.is_null() {
                anyhow::bail!("create_sender returned null");
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

    fn send(&mut self, ctx: &mut dyn SystemContext, data: &[u8]) {
        unsafe {
            with_context(ctx, || {
                (self.on_app_data_fn)(self.instance, data.as_ptr(), data.len());
            });
        }
    }
}

/// Load a C++ sender library from the given path and wrap it as a Rust TransportProtocol.
pub fn load_cpp_sender<P: AsRef<Path>>(path: P) -> anyhow::Result<Box<dyn TransportProtocol>> {
    let lib = unsafe { Library::new(path.as_ref()) }
        .with_context(|| format!("failed to load C++ sender library {:?}", path.as_ref()))?;
    let cpp = CppTransportProtocol::new(lib)?;
    Ok(Box::new(cpp))
}



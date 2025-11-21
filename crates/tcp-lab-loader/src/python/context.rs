use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::cell::RefCell;
use tcp_lab_abstract::SystemContext;

use super::adapter;

// Thread-local storage to hold the reference to SystemContext during callbacks
thread_local! {
    // We use 'static here to satisfy TLS requirements, but we manually manage validity.
    static CURRENT_CONTEXT: RefCell<Option<*mut (dyn SystemContext + 'static)>> = RefCell::new(None);
}

/// Execute the given closure with the SystemContext active in TLS.
pub fn with_context<F, R>(ctx: &mut dyn SystemContext, f: F) -> R
where
    F: FnOnce() -> R,
{
    let ptr = ctx as *mut dyn SystemContext;
    // Transmute to extend lifetime to 'static for storage in TLS.
    // SAFETY: We guarantee that `ptr` is valid for the duration of `f()`
    // and we clear it immediately after.
    let static_ptr = unsafe {
        std::mem::transmute::<*mut dyn SystemContext, *mut (dyn SystemContext + 'static)>(ptr)
    };

    CURRENT_CONTEXT.with(|c| {
        *c.borrow_mut() = Some(static_ptr);
    });

    let result = f();

    CURRENT_CONTEXT.with(|c| {
        *c.borrow_mut() = None;
    });

    result
}

fn use_context<F, R>(f: F) -> PyResult<R>
where
    F: FnOnce(&mut dyn SystemContext) -> PyResult<R>,
{
    CURRENT_CONTEXT.with(|c| {
        if let Some(ptr) = *c.borrow() {
            // SAFETY: The pointer is valid because `with_context` ensures it
            // stays valid for the duration of the callback.
            let ctx = unsafe { &mut *ptr };
            f(ctx)
        } else {
            Err(PyRuntimeError::new_err(
                "SystemContext not active (called outside callback?)",
            ))
        }
    })
}

/// The SystemContext implementation exposed to Python.
/// This class has no state; it proxies calls to the TLS context.
#[pyclass(name = "SystemContextImpl")]
pub struct PySystemContext;

#[pymethods]
impl PySystemContext {
    #[new]
    pub fn new() -> Self {
        PySystemContext
    }

    fn send_packet(&self, packet: &Bound<'_, PyAny>) -> PyResult<()> {
        let pkt = adapter::from_py_packet(packet)?;
        use_context(|ctx| {
            ctx.send_packet(pkt);
            Ok(())
        })
    }

    fn start_timer(&self, delay_ms: u64, timer_id: u32) -> PyResult<()> {
        use_context(|ctx| {
            ctx.start_timer(delay_ms, timer_id);
            Ok(())
        })
    }

    fn cancel_timer(&self, timer_id: u32) -> PyResult<()> {
        use_context(|ctx| {
            ctx.cancel_timer(timer_id);
            Ok(())
        })
    }

    fn deliver_data(&self, data: &[u8]) -> PyResult<()> {
        use_context(|ctx| {
            ctx.deliver_data(data);
            Ok(())
        })
    }

    fn log(&self, message: &str) -> PyResult<()> {
        use_context(|ctx| {
            ctx.log(message);
            Ok(())
        })
    }

    fn now(&self) -> PyResult<u64> {
        use_context(|ctx| Ok(ctx.now()))
    }

    fn record_metric(&self, name: &str, value: f64) -> PyResult<()> {
        use_context(|ctx| {
            ctx.record_metric(name, value);
            Ok(())
        })
    }
}

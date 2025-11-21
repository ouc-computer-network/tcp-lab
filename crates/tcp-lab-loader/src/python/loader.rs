use anyhow::{Context, Result};
use pyo3::prelude::*;
use tcp_lab_abstract::{Packet, SystemContext, TransportProtocol};

use super::adapter;
use super::context::{with_context, PySystemContext};
use super::environment::PythonEnvironment;

pub struct PythonTransportProtocol {
    instance: Py<PyAny>,
}

impl PythonTransportProtocol {
    pub fn new(
        module_name: &str,
        class_name: &str,
        env: Option<&PythonEnvironment>,
    ) -> Result<Self> {
        Python::attach(|py| {
            if let Some(env) = env {
                env.inject(py)
                    .map_err(|e| anyhow::anyhow!("Failed to activate Python environment: {}", e))?;
            }

            let module = py
                .import(module_name)
                .with_context(|| format!("Failed to import Python module '{}'", module_name))?;

            let cls = module.getattr(class_name).with_context(|| {
                format!(
                    "Failed to find class '{}' in module '{}'",
                    class_name, module_name
                )
            })?;

            let instance = cls
                .call0()
                .with_context(|| format!("Failed to instantiate class '{}'", class_name))?;

            Ok(Self {
                instance: instance.into(),
            })
        })
    }
}

impl TransportProtocol for PythonTransportProtocol {
    fn init(&mut self, ctx: &mut dyn SystemContext) {
        with_context(ctx, || {
            Python::attach(|py| {
                let py_ctx = PySystemContext::new();
                if let Err(e) = self.instance.call_method1(py, "init", (py_ctx,)) {
                    eprintln!("Python init failed: {}", e);
                    e.print(py);
                }
            })
        })
    }

    fn on_packet(&mut self, ctx: &mut dyn SystemContext, packet: Packet) {
        with_context(ctx, || {
            Python::attach(|py| {
                let py_ctx = PySystemContext::new();
                let py_packet = match adapter::to_py_packet(py, packet) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("Failed to convert packet to Python: {}", e);
                        return;
                    }
                };

                if let Err(e) = self
                    .instance
                    .call_method1(py, "on_packet", (py_ctx, py_packet))
                {
                    eprintln!("Python on_packet failed: {}", e);
                    e.print(py);
                }
            })
        })
    }

    fn on_timer(&mut self, ctx: &mut dyn SystemContext, timer_id: u32) {
        with_context(ctx, || {
            Python::attach(|py| {
                let py_ctx = PySystemContext::new();
                if let Err(e) = self
                    .instance
                    .call_method1(py, "on_timer", (py_ctx, timer_id))
                {
                    eprintln!("Python on_timer failed: {}", e);
                    e.print(py);
                }
            })
        })
    }

    fn on_app_data(&mut self, ctx: &mut dyn SystemContext, data: &[u8]) {
        with_context(ctx, || {
            Python::attach(|py| {
                let py_ctx = PySystemContext::new();
                let py_data = pyo3::types::PyBytes::new(py, data);
                if let Err(e) = self
                    .instance
                    .call_method1(py, "on_app_data", (py_ctx, py_data))
                {
                    eprintln!("Python on_app_data failed: {}", e);
                    e.print(py);
                }
            })
        })
    }
}

pub fn load_protocol(
    module: &str,
    class: &str,
    env: Option<&PythonEnvironment>,
) -> Result<Box<dyn TransportProtocol>> {
    let protocol = PythonTransportProtocol::new(module, class, env)?;
    Ok(Box::new(protocol))
}

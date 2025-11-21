mod builtin;
#[cfg(feature = "cpp")]
pub mod cpp;
#[cfg(feature = "java")]
mod java;
#[cfg(feature = "python")]
pub mod python;
pub mod spec;

use anyhow::Result;
use std::path::PathBuf;
use tcp_lab_abstract::TransportProtocol;

#[cfg(feature = "java")]
use anyhow::Context;
#[cfg(feature = "java")]
use java::create_jvm;
#[cfg(feature = "java")]
use jni::JavaVM;
#[cfg(feature = "java")]
use std::sync::Arc;

#[cfg(feature = "python")]
use python::environment::PythonEnvironment;

#[cfg(not(feature = "python"))]
#[derive(Clone, Debug)]
struct PythonEnvironment;

#[cfg(not(feature = "java"))]
type JavaVmHandle = ();
#[cfg(feature = "java")]
type JavaVmHandle = Arc<JavaVM>;

/// Built-in Rust implementations that can be used without loading external code.
#[derive(Clone, Copy, Debug)]
pub enum BuiltinProtocol {
    Rdt2Sender,
    Rdt2Receiver,
}

/// Describes how to obtain a transport protocol implementation.
pub enum ProtocolDescriptor {
    BuiltIn(BuiltinProtocol),
    Java { class_name: String },
    Python { module: String, class_name: String },
    Cpp { library_path: PathBuf },
    Rust(Box<dyn TransportProtocol>),
}

/// Pair of protocol descriptors used by the loader.
#[derive(Default)]
pub struct LoaderRequest {
    pub sender: Option<ProtocolDescriptor>,
    pub receiver: Option<ProtocolDescriptor>,
}

/// Python environment configuration. Allows pointing to a uv-managed project
/// as well as adding ad-hoc search paths.
#[derive(Default, Clone)]
pub struct PythonConfig {
    uv_project_root: Option<PathBuf>,
    extra_paths: Vec<PathBuf>,
}

impl PythonConfig {
    pub fn with_uv_project(mut self, root: impl Into<PathBuf>) -> Self {
        self.uv_project_root = Some(root.into());
        self
    }

    pub fn add_sys_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.extra_paths.push(path.into());
        self
    }
}

/// Builder for the loader. Allows configuring shared state (e.g. JVM, uv env).
pub struct LoaderBuilder {
    java_classpath: Option<String>,
    python: Option<PythonConfig>,
}

impl Default for LoaderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LoaderBuilder {
    pub fn new() -> Self {
        Self {
            java_classpath: None,
            python: None,
        }
    }

    pub fn java_classpath(mut self, classpath: impl Into<String>) -> Self {
        self.java_classpath = Some(classpath.into());
        self
    }

    pub fn python_config(mut self, config: PythonConfig) -> Self {
        self.python = Some(config);
        self
    }

    pub fn build(self) -> Result<ProtocolLoader> {
        #[cfg(feature = "java")]
        let java_vm = init_java(self.java_classpath)?;
        #[cfg(not(feature = "java"))]
        {
            let _ = init_java(self.java_classpath)?;
        }

        #[cfg(feature = "python")]
        let python_env = init_python(self.python)?;
        #[cfg(not(feature = "python"))]
        {
            let _ = init_python(self.python)?;
        }

        Ok(ProtocolLoader {
            #[cfg(feature = "java")]
            java_vm,
            #[cfg(feature = "python")]
            python_env,
        })
    }
}

fn init_java(classpath: Option<String>) -> Result<Option<JavaVmHandle>> {
    #[cfg(feature = "java")]
    {
        if let Some(cp) = classpath {
            let vm = create_jvm(&cp)?;
            Ok(Some(vm))
        } else {
            Ok(None)
        }
    }
    #[cfg(not(feature = "java"))]
    {
        if classpath.is_some() {
            anyhow::bail!("`java` feature disabled but Java classpath provided");
        }
        Ok(None)
    }
}

fn init_python(config: Option<PythonConfig>) -> Result<Option<PythonEnvironment>> {
    #[cfg(feature = "python")]
    {
        if let Some(config) = config {
            let env = if let Some(root) = config.uv_project_root {
                PythonEnvironment::from_uv(root, &config.extra_paths)?
            } else if !config.extra_paths.is_empty() {
                PythonEnvironment::from_paths(config.extra_paths)
            } else {
                return Ok(None);
            };

            // Set PYTHONHOME environment variable if available
            if let Some(python_home) = env.python_home() {
                unsafe {
                    std::env::set_var("PYTHONHOME", python_home);
                }
            }

            Ok(Some(env))
        } else {
            Ok(None)
        }
    }
    #[cfg(not(feature = "python"))]
    {
        if config.is_some() {
            anyhow::bail!("`python` feature disabled but python config provided");
        }
        Ok(None)
    }
}

/// Loader capable of instantiating sender/receiver implementations across languages.
pub struct ProtocolLoader {
    #[cfg(feature = "java")]
    java_vm: Option<JavaVmHandle>,
    #[cfg(feature = "python")]
    python_env: Option<PythonEnvironment>,
}

impl ProtocolLoader {
    pub fn builder() -> LoaderBuilder {
        LoaderBuilder::new()
    }

    pub fn load_pair(
        &self,
        request: LoaderRequest,
    ) -> Result<(Box<dyn TransportProtocol>, Box<dyn TransportProtocol>)> {
        let sender = match request.sender {
            Some(desc) => self.load(desc)?,
            None => builtin::default_sender(),
        };
        let receiver = match request.receiver {
            Some(desc) => self.load(desc)?,
            None => builtin::default_receiver(),
        };
        Ok((sender, receiver))
    }

    pub fn load(&self, descriptor: ProtocolDescriptor) -> Result<Box<dyn TransportProtocol>> {
        match descriptor {
            ProtocolDescriptor::BuiltIn(builtin) => Ok(match builtin {
                BuiltinProtocol::Rdt2Sender => builtin::rdt2_sender(),
                BuiltinProtocol::Rdt2Receiver => builtin::rdt2_receiver(),
            }),
            ProtocolDescriptor::Java { class_name } => self.load_java(&class_name),
            ProtocolDescriptor::Python { module, class_name } => {
                self.load_python(&module, &class_name)
            }
            ProtocolDescriptor::Cpp { library_path } => self.load_cpp(&library_path),
            ProtocolDescriptor::Rust(protocol) => Ok(protocol),
        }
    }

    #[cfg(feature = "java")]
    fn load_java(&self, class_name: &str) -> Result<Box<dyn TransportProtocol>> {
        let vm = self
            .java_vm
            .as_ref()
            .context("JVM not configured; call LoaderBuilder::java_classpath first")?;
        java::load_protocol(vm, class_name)
    }

    #[cfg(not(feature = "java"))]
    fn load_java(&self, _class_name: &str) -> Result<Box<dyn TransportProtocol>> {
        anyhow::bail!("Java support disabled at compile time");
    }

    #[cfg(feature = "python")]
    fn load_python(&self, module: &str, class_name: &str) -> Result<Box<dyn TransportProtocol>> {
        python::loader::load_protocol(module, class_name, self.python_env.as_ref())
    }

    #[cfg(not(feature = "python"))]
    fn load_python(&self, _module: &str, _class_name: &str) -> Result<Box<dyn TransportProtocol>> {
        anyhow::bail!("Python support disabled at compile time");
    }

    #[cfg(feature = "cpp")]
    fn load_cpp(&self, path: &PathBuf) -> Result<Box<dyn TransportProtocol>> {
        cpp::loader::load_protocol(path)
    }

    #[cfg(not(feature = "cpp"))]
    fn load_cpp(&self, _path: &PathBuf) -> Result<Box<dyn TransportProtocol>> {
        anyhow::bail!("C++ support disabled at compile time");
    }
}

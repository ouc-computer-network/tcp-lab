use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tcp_lab_loader::spec::{builtin_by_name, parse_python_spec};
use tcp_lab_loader::{LoaderRequest, ProtocolDescriptor, ProtocolLoader, PythonConfig};
use tcp_lab_simulator::{scenario_runner, SimulationReport};
use tracing::info;

#[derive(Parser, Debug)]
#[command(author, version, about = "Headless grader for TCP Lab scenarios")]
struct Args {
    /// Path to the scenario TOML file to execute.
    #[arg(long)]
    scenario: String,

    /// Java classpath used when loading JVM-based implementations.
    #[arg(long)]
    classpath: Option<String>,

    #[arg(long)]
    java_sender: Option<String>,
    #[arg(long)]
    java_receiver: Option<String>,

    #[arg(long)]
    python_sender: Option<String>,
    #[arg(long)]
    python_receiver: Option<String>,

    /// Root directory of the uv-managed Python project.
    #[arg(long)]
    python_uv_project: Option<PathBuf>,

    /// Extra path to insert into Python sys.path (in addition to uv).
    #[arg(long)]
    python_path: Option<PathBuf>,

    #[arg(long)]
    cpp_sender_lib: Option<PathBuf>,
    #[arg(long)]
    cpp_receiver_lib: Option<PathBuf>,

    #[arg(long)]
    builtin_sender: Option<String>,
    #[arg(long)]
    builtin_receiver: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    tracing_subscriber::fmt::init();
    info!("tcp-lab-eval-host starting...");

    let loader = build_loader(&args)?;
    let request = LoaderRequest {
        sender: args.sender_descriptor()?,
        receiver: args.receiver_descriptor()?,
    };

    let (sender, receiver) = loader.load_pair(request)?;
    let report = scenario_runner::run_scenario(&args.scenario, sender, receiver)?;
    log_summary(&report);
    Ok(())
}

fn build_loader(args: &Args) -> Result<ProtocolLoader> {
    let mut builder = ProtocolLoader::builder();
    if let Some(cp) = &args.classpath {
        builder = builder.java_classpath(cp.clone());
    }

    if args.python_uv_project.is_some() || args.python_path.is_some() {
        let mut cfg = PythonConfig::default();
        if let Some(root) = &args.python_uv_project {
            cfg = cfg.with_uv_project(root.clone());
        }
        if let Some(path) = &args.python_path {
            cfg = cfg.add_sys_path(path.clone());
        }
        builder = builder.python_config(cfg);
    }

    builder.build()
}

fn log_summary(report: &SimulationReport) {
    info!(
        "Simulation duration: {} ms | packets sent: {} | deliveries: {}",
        report.duration_ms,
        report.sender_packet_count,
        report.delivered_data.len()
    );
}

impl Args {
    fn sender_descriptor(&self) -> Result<Option<ProtocolDescriptor>> {
        self.resolve_descriptor(
            &self.java_sender,
            &self.python_sender,
            self.cpp_sender_lib.as_ref(),
            self.builtin_sender.as_deref(),
            true,
        )
    }

    fn receiver_descriptor(&self) -> Result<Option<ProtocolDescriptor>> {
        self.resolve_descriptor(
            &self.java_receiver,
            &self.python_receiver,
            self.cpp_receiver_lib.as_ref(),
            self.builtin_receiver.as_deref(),
            false,
        )
    }

    fn resolve_descriptor(
        &self,
        java: &Option<String>,
        python: &Option<String>,
        cpp: Option<&PathBuf>,
        builtin: Option<&str>,
        is_sender: bool,
    ) -> Result<Option<ProtocolDescriptor>> {
        if let Some(class_name) = java {
            return Ok(Some(ProtocolDescriptor::Java {
                class_name: class_name.clone(),
            }));
        }

        if let Some(spec) = python {
            let (module, class_name) = parse_python_spec(spec)?;
            return Ok(Some(ProtocolDescriptor::Python { module, class_name }));
        }

        if let Some(path) = cpp {
            return Ok(Some(ProtocolDescriptor::Cpp {
                library_path: path.clone(),
            }));
        }

        if let Some(name) = builtin {
            let builtin = builtin_by_name(name, is_sender)?;
            return Ok(Some(ProtocolDescriptor::BuiltIn(builtin)));
        }

        Ok(None)
    }
}

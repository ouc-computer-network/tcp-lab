use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

use tcp_lab_abstract::{SimConfig, TestAction, TestScenario, TransportProtocol};
use tcp_lab_loader::spec::{builtin_by_name, parse_python_spec};
use tcp_lab_loader::{LoaderRequest, ProtocolDescriptor, ProtocolLoader, PythonConfig};
use tcp_lab_simulator::tui::{MemoryLogBuffer, TuiApp};
use tcp_lab_simulator::{SimulationReport, Simulator, encda, scenario_runner};

#[derive(Parser, Debug)]
#[command(author, version, about = "Interactive TCP Lab simulator")]
struct Args {
    /// Load a scenario from disk.
    #[arg(long)]
    scenario: Option<PathBuf>,

    /// Launch the terminal UI visualizer.
    #[arg(long, default_value_t = false)]
    tui: bool,

    /// JVM classpath used when loading Java implementations.
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

    /// Extra path added to Python sys.path (in addition to uv).
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

    /// Write a JSON trace of the finished simulation.
    #[arg(long)]
    trace_out: Option<PathBuf>,

    /// Play an encrypted ENCDA.tcp trace (mutually exclusive with --scenario).
    #[arg(long)]
    encda: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let _log_guard = init_logging(args.tui);
    info!("tcp-lab-sim-cli starting…");

    let loader = args.build_loader()?;
    let request = args.loader_request()?;
    let (sender, receiver) = loader.load_pair(request)?;

    if args.scenario.is_some() && args.encda.is_some() {
        anyhow::bail!("--scenario and --encda cannot be used together");
    }

    let report = if let Some(path) = &args.encda {
        let dataset = encda::load_from_file(path)?;
        run_encda_sim(args.tui, dataset, sender, receiver)?
    } else if let Some(path) = &args.scenario {
        if args.tui {
            let scenario = load_scenario(path)?;
            run_scenario_tui(scenario, sender, receiver)?
        } else {
            let scenario_path = path
                .to_str()
                .context("Scenario path contains invalid UTF-8")?;
            scenario_runner::run_scenario(scenario_path, sender, receiver)?
        }
    } else {
        run_default_sim(args.tui, sender, receiver)?
    };

    if let Some(trace_path) = &args.trace_out {
        write_trace(trace_path, &report)?;
    }

    Ok(())
}

impl Args {
    fn loader_request(&self) -> Result<LoaderRequest> {
        Ok(LoaderRequest {
            sender: self.resolve_descriptor(
                &self.java_sender,
                &self.python_sender,
                self.cpp_sender_lib.as_ref(),
                self.builtin_sender.as_deref(),
                true,
            )?,
            receiver: self.resolve_descriptor(
                &self.java_receiver,
                &self.python_receiver,
                self.cpp_receiver_lib.as_ref(),
                self.builtin_receiver.as_deref(),
                false,
            )?,
        })
    }

    fn build_loader(&self) -> Result<ProtocolLoader> {
        let mut builder = ProtocolLoader::builder();
        if let Some(cp) = &self.classpath {
            builder = builder.java_classpath(cp.clone());
        }

        if self.python_uv_project.is_some() || self.python_path.is_some() {
            let mut cfg = PythonConfig::default();
            if let Some(root) = &self.python_uv_project {
                cfg = cfg.with_uv_project(root.clone());
            }
            if let Some(extra) = &self.python_path {
                cfg = cfg.add_sys_path(extra.clone());
            }
            builder = builder.python_config(cfg);
        }

        builder.build()
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

fn init_logging(use_tui: bool) -> Option<MemoryLogBuffer> {
    if use_tui {
        let buffer = MemoryLogBuffer::new();
        let writer = buffer.clone();
        tracing_subscriber::fmt()
            .with_writer(move || writer.clone())
            .with_ansi(false)
            .init();
        Some(buffer)
    } else {
        tracing_subscriber::fmt::init();
        None
    }
}

fn run_default_sim(
    use_tui: bool,
    sender: Box<dyn TransportProtocol>,
    receiver: Box<dyn TransportProtocol>,
) -> Result<SimulationReport> {
    let mut sim = build_default_sim(sender, receiver);
    if use_tui {
        let mut app = TuiApp::new(sim, None);
        app.run()?;
        let sim = app.into_simulator();
        Ok(sim.export_report())
    } else {
        info!("Starting default headless simulation…");
        sim.run_until_complete();
        info!("Simulation complete.");
        Ok(sim.export_report())
    }
}

fn build_default_sim(
    sender: Box<dyn TransportProtocol>,
    receiver: Box<dyn TransportProtocol>,
) -> Simulator {
    let config = SimConfig {
        loss_rate: 0.1,
        min_latency: 100,
        max_latency: 500,
        seed: 42,
        ..Default::default()
    };
    let mut sim = Simulator::new(config, sender, receiver);
    sim.schedule_app_send(1000, b"Packet 1".to_vec());
    sim.schedule_app_send(2000, b"Packet 2".to_vec());
    sim.schedule_app_send(3000, b"Packet 3".to_vec());
    sim
}

fn run_scenario_tui(
    scenario: TestScenario,
    sender: Box<dyn TransportProtocol>,
    receiver: Box<dyn TransportProtocol>,
) -> Result<SimulationReport> {
    let mut config = SimConfig::default();
    scenario.config.apply_to(&mut config);
    let mut sim = Simulator::new(config, sender, receiver);
    configure_actions(&mut sim, &scenario.actions);

    let mut app = TuiApp::new(sim, Some(scenario.name.clone()));
    app.run()?;
    let sim = app.into_simulator();
    Ok(sim.export_report())
}

fn run_encda_sim(
    use_tui: bool,
    dataset: encda::EncdaDataset,
    sender: Box<dyn TransportProtocol>,
    receiver: Box<dyn TransportProtocol>,
) -> Result<SimulationReport> {
    let mut sim = build_default_sim(sender, receiver);
    for (idx, chunk) in dataset.groups.iter().enumerate() {
        let time = (idx as u64) * 10;
        sim.schedule_app_send(time, chunk.clone());
    }
    if use_tui {
        let mut app = TuiApp::new(sim, Some("ENCDA Trace".to_string()));
        app.run()?;
        Ok(app.into_simulator().export_report())
    } else {
        info!(
            "Running ENCDA trace with {} groups of {} bytes",
            dataset.groups.len(),
            dataset.group_size
        );
        sim.run_until_complete();
        Ok(sim.export_report())
    }
}

fn configure_actions(sim: &mut Simulator, actions: &[TestAction]) {
    for action in actions {
        match action {
            TestAction::AppSend { time, data } => {
                sim.schedule_app_send(*time, data.as_bytes().to_vec());
            }
            TestAction::DropNextFromSenderSeq { seq } => {
                sim.add_drop_sender_seq_once(*seq);
            }
            TestAction::DropNextFromReceiverAck { ack } => {
                sim.add_drop_receiver_ack_once(*ack);
            }
        }
    }
}

fn load_scenario(path: &Path) -> Result<TestScenario> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read scenario file {}", path.display()))?;
    let scenario: TestScenario =
        toml::from_str(&content).context("Failed to parse scenario file")?;
    Ok(scenario)
}

fn write_trace(path: &Path, report: &SimulationReport) -> Result<()> {
    let data = serde_json::to_vec_pretty(report).context("Failed to serialize simulation trace")?;
    fs::write(path, &data)
        .with_context(|| format!("Failed to write trace file {}", path.display()))?;
    Ok(())
}

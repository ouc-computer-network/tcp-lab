pub mod cpp;
pub mod examples;
pub mod java_loader;
pub mod python;
pub mod runner;
pub mod tui;

use crate::examples::{SimpleReceiver, SimpleSender};
use crate::tui::{MemoryLogBuffer, TuiApp};
use clap::Parser;
use std::fs;
use tcp_lab_core::{SimConfig, Simulator, TestAction, TestScenario, TransportProtocol};
use tcp_lab_ffi::ensure_linked;
use tracing::info;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub config: Option<String>,

    #[arg(long, default_value_t = false)]
    pub tui: bool,

    #[arg(long)]
    pub java_sender: Option<String>,

    #[arg(long)]
    pub java_receiver: Option<String>,

    #[arg(long, default_value = ".")]
    pub classpath: String,

    #[arg(long)]
    pub test_scenario: Option<String>,

    /// Path to a C++ sender shared library (e.g. libgbn_sender.so / .dylib / .dll).
    #[arg(long)]
    pub cpp_sender_lib: Option<String>,

    /// Path to a C++ receiver shared library (same ABI as sender, but different logic).
    #[arg(long)]
    pub cpp_receiver_lib: Option<String>,

    /// Python module and class for sender (e.g. "examples.rdt3_sender.Rdt3Sender")
    #[arg(long)]
    pub python_sender: Option<String>,

    /// Python module and class for receiver (e.g. "examples.rdt3_receiver.Rdt3Receiver")
    #[arg(long)]
    pub python_receiver: Option<String>,

    /// Additional path to add to Python sys.path
    #[arg(long)]
    pub python_path: Option<String>,
}

pub fn run(args: Args, rust_sender: Option<Box<dyn TransportProtocol>>, rust_receiver: Option<Box<dyn TransportProtocol>>) -> anyhow::Result<()> {
    ensure_linked();

    // Setup Logging
    if args.tui {
        let buffer = MemoryLogBuffer::new();
        let writer_buffer = buffer.clone();

        tracing_subscriber::fmt()
            .with_writer(move || writer_buffer.clone())
            .with_ansi(false)
            .init();
    } else {
        tracing_subscriber::fmt::init();
    }

    info!("TCP Lab Simulator starting...");

    // Init JVM if needed
    let jvm = if args.java_sender.is_some() || args.java_receiver.is_some() {
        info!("Initializing JVM with classpath: {}", args.classpath);
        Some(java_loader::create_jvm(&args.classpath)?)
    } else {
        None
    };

    // Helper to parse "module.class"
    let parse_py_arg = |s: &str| -> anyhow::Result<(String, String)> {
        if let Some((module, class)) = s.rsplit_once('.') {
            Ok((module.to_string(), class.to_string()))
        } else {
            anyhow::bail!(
                "Invalid python argument format '{}'. Expected 'module.Class'",
                s
            );
        }
    };

    // Setup Protocols
    let sender: Box<dyn TransportProtocol> = if let Some(cls) = &args.java_sender {
        info!("Loading Java Sender: {}", cls);
        java_loader::load_protocol(jvm.as_ref().unwrap(), cls)?
    } else if let Some(path) = &args.cpp_sender_lib {
        info!("Loading C++ Sender from {:?}", path);
        cpp::loader::load_protocol(path)?
    } else if let Some(py_arg) = &args.python_sender {
        let (module, class) = parse_py_arg(py_arg)?;
        info!("Loading Python Sender: {}.{}", module, class);
        python::loader::load_protocol(&module, &class, args.python_path.as_deref())?
    } else if let Some(s) = rust_sender {
        info!("Using provided Rust Sender");
        s
    } else {
        Box::new(SimpleSender::default())
    };

    let receiver: Box<dyn TransportProtocol> = if let Some(cls) = &args.java_receiver {
        info!("Loading Java Receiver: {}", cls);
        java_loader::load_protocol(jvm.as_ref().unwrap(), cls)?
    } else if let Some(path) = &args.cpp_receiver_lib {
        info!("Loading C++ Receiver from {:?}", path);
        cpp::loader::load_protocol(path)?
    } else if let Some(py_arg) = &args.python_receiver {
        let (module, class) = parse_py_arg(py_arg)?;
        info!("Loading Python Receiver: {}.{}", module, class);
        python::loader::load_protocol(&module, &class, args.python_path.as_deref())?
    } else if let Some(r) = rust_receiver {
        info!("Using provided Rust Receiver");
        r
    } else {
        Box::new(SimpleReceiver::default())
    };

    // If a test scenario is provided, either run headless grader or visualize via TUI
    if let Some(scenario_path) = &args.test_scenario {
        if args.tui {
            // Load scenario and run it through the TUI (no assertions, purely for visualization)
            let content = fs::read_to_string(scenario_path)?;
            let scenario: TestScenario = toml::from_str(&content)?;

            let mut config = SimConfig::default();
            scenario.config.apply_to(&mut config);

            let mut sim = Simulator::new(config, sender, receiver);
            for action in &scenario.actions {
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

            let mut app = TuiApp::new(sim, Some(scenario.name.clone()));
            app.run()?;
            return Ok(());
        } else {
            // Run automated graded test (headless)
            runner::run_scenario(scenario_path, sender, receiver)?;
            return Ok(());
        }
    }

    // Setup Default Simulation (if not testing)
    let config = SimConfig {
        loss_rate: 0.1,
        min_latency: 100,
        max_latency: 500,
        seed: 42,
        ..Default::default()
    };

    let mut sim = Simulator::new(config, sender, receiver);

    // Schedule some data to be sent (Default behavior)
    sim.schedule_app_send(1000, b"Packet 1".to_vec());
    sim.schedule_app_send(2000, b"Packet 2".to_vec());
    sim.schedule_app_send(3000, b"Packet 3".to_vec());

    if args.tui {
        let mut app = TuiApp::new(sim, None);
        app.run()?;
    } else {
        // Run headless
        info!("Starting simulation loop...");
        sim.run_until_complete();
        info!("Simulation complete.");
    }

    Ok(())
}
mod examples;
mod tui;
mod java_loader;
mod runner;

use clap::Parser;
use tracing::info;
use tcp_lab_core::{Simulator, SimConfig, TransportProtocol};
use crate::examples::{SimpleSender, SimpleReceiver};
use crate::tui::{TuiApp, MemoryLogBuffer};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: Option<String>,

    #[arg(long, default_value_t = false)]
    tui: bool,
    
    #[arg(long)]
    java_sender: Option<String>,

    #[arg(long)]
    java_receiver: Option<String>,
    
    #[arg(long, default_value = ".")]
    classpath: String,

    #[arg(long)]
    test_scenario: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Setup Logging
    let log_buffer = if args.tui {
        let buffer = MemoryLogBuffer::new();
        let writer_buffer = buffer.clone();
        
        tracing_subscriber::fmt()
            .with_writer(move || writer_buffer.clone())
            .with_ansi(false) 
            .init();
        Some(buffer)
    } else {
        tracing_subscriber::fmt::init();
        None
    };

    info!("TCP Lab Simulator starting...");

    // Init JVM if needed
    let jvm = if args.java_sender.is_some() || args.java_receiver.is_some() {
        info!("Initializing JVM with classpath: {}", args.classpath);
        Some(java_loader::create_jvm(&args.classpath)?)
    } else {
        None
    };

    // Setup Protocols
    let sender: Box<dyn TransportProtocol> = if let Some(cls) = &args.java_sender {
        info!("Loading Java Sender: {}", cls);
        java_loader::load_java_protocol(jvm.as_ref().unwrap(), cls)?
    } else {
        Box::new(SimpleSender::default())
    };

    let receiver: Box<dyn TransportProtocol> = if let Some(cls) = &args.java_receiver {
        info!("Loading Java Receiver: {}", cls);
        java_loader::load_java_protocol(jvm.as_ref().unwrap(), cls)?
    } else {
        Box::new(SimpleReceiver::default())
    };

    if let Some(scenario) = args.test_scenario {
        // Run automated test
        runner::run_scenario(&scenario, sender, receiver)?;
        return Ok(());
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
        if let Some(buffer) = log_buffer {
            let mut app = TuiApp::new(sim, buffer);
            app.run()?;
        }
    } else {
        // Run headless
        info!("Starting simulation loop...");
        sim.run_until_complete();
        info!("Simulation complete.");
    }

    Ok(())
}

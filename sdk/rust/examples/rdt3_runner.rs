use clap::Parser;
use tcp_lab_cli::{run, Args};
use tcp_lab_rust::examples::rdt3_receiver::Rdt3Receiver;
use tcp_lab_rust::examples::rdt3_sender::Rdt3Sender;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    info!("Starting RDT3 Runner Example");

    let sender = Box::new(Rdt3Sender::new());
    let receiver = Box::new(Rdt3Receiver::new());

    run(args, Some(sender), Some(receiver))
}
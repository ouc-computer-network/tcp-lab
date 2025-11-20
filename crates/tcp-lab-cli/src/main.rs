use clap::Parser;
use tcp_lab_cli::{run, Args};
use tcp_lab_core::TransportProtocol;
use tracing::info;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    #[command(flatten)]
    args: Args,

    /// Use built-in Rust RDT3 Sender example
    #[arg(long, default_value_t = false)]
    rust_rdt3_sender: bool,

    /// Use built-in Rust RDT3 Receiver example
    #[arg(long, default_value_t = false)]
    rust_rdt3_receiver: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli_args = CliArgs::parse();

    // Legacy support for built-in Rust examples is temporarily disabled to break cyclic dependency
    // Users should use the `rdt3_runner` example in `sdk/rust` instead.
    if cli_args.rust_rdt3_sender || cli_args.rust_rdt3_receiver {
        anyhow::bail!("The --rust-rdt3-sender and --rust-rdt3-receiver flags are deprecated. Please use `cargo run --example rdt3_runner` in `sdk/rust` instead.");
    }

    let rust_sender: Option<Box<dyn TransportProtocol>> = None;
    let rust_receiver: Option<Box<dyn TransportProtocol>> = None;

    run(cli_args.args, rust_sender, rust_receiver)
}

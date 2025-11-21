# Rust SDK

This crate exposes two things:

1. `tcp_lab_rust_sdk::rdt1::{sender, receiver}` – a minimal RDT1 reference implementation.
2. `tcp_lab_rust_sdk::checksum::internet_checksum` – a 16-bit ones' complement helper for future RDT2+ assignments.

## Usage

Add this crate as a workspace member and depend on it from your own protocol crate:

```toml
[dependencies]
tcp-lab-rust-sdk = { path = "../sdk/rust" }
```

Implement the `tcp_lab_abstract::TransportProtocol` trait in your crate (or start from the RDT1 structs) and then embed it in a simulator:

```rust
use tcp_lab_rust_sdk::rdt1::{Rdt1Sender, Rdt1Receiver};
use tcp_lab_simulator::{Simulator, SimConfig};

fn main() {
    let mut sim = Simulator::new(SimConfig::default(), Box::new(Rdt1Sender::default()), Box::new(Rdt1Receiver::default()));
    sim.schedule_app_send(0, b"hello".to_vec());
    sim.run_until_complete();
}
```

For a ready-to-run demo, execute:

```
cargo run -p tcp-lab-rust-sdk --example rdt1_runner
```

### Exporting a Protocol for the CLI

Once you implement RDT2, expose a constructor that returns a `Box<dyn TransportProtocol>`. Then, in a binary (or test harness) you can connect it with real scenarios using `tcp-lab-simulator` just like the example.

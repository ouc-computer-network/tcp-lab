# TCP Lab Workspace

This repository hosts the next generation of the TCP teaching lab. It is built as a Cargo workspace and separates responsibilities into a few dedicated crates so that student code, grading logic, and visualization tools stay loosely coupled.

## Components

| Crate | Purpose |
|-------|---------|
| `tcp-lab-abstract` | Trait definitions (`TransportProtocol`, `SystemContext`) and shared data structs (packets, scenarios, simulator config). Every language binding depends on this. |
| `tcp-lab-loader` | Feature-gated bridges that load student implementations from Rust, Java (`--features java`), Python/uv (`--features python`), and C++ (`--features cpp`). Also exposes built-in reference protocols (RDT2 stop-and-wait). |
| `tcp-lab-simulator` | Deterministic discrete-event simulator with optional TUI. Houses the scenario runner, link-space visualization, ENCDA.tcp decoder, and JSON trace exporter. |
| `tcp-lab-sim-cli` | Developer-facing CLI for ad-hoc runs and visualization. Uses the loader to bootstrap student code, can replay TOML scenarios or encrypted `ENCDA.tcp` traces, and exports `SimulationReport` JSON via `--trace-out`. |
| `tcp-lab-eval-host` | Headless grader CLI. Reads scenario TOML, loads sender/receiver via the loader, and exits with success/failure for use in autograders/CI. |

The legacy Java project lives in `legacy_java/` for reference; its encrypted trace file (`ENCDA.tcp`) can be visualized using the sim CLI.

## Quick Start

1. **Rust toolchain**: install Rust 1.72+; `cargo` drives the entire workspace.
2. **Optional bridges**: enable Java/Python/C++ loaders only when you need them:
   ```bash
   # Example: enable python loader + TUI support when running the sim CLI
   cargo run -p tcp-lab-sim-cli --features "python" -- --python-sender my.module.Sender
   ```
3. **Visualize a scenario**:
   ```bash
   cargo run -p tcp-lab-sim-cli -- --tui \
       --scenario tests/scenarios/rdt2_basic.toml \
       --trace-out traces/rdt2_basic.json
   ```
4. **Replay the legacy ENCDA trace**:
   ```bash
   cargo run -p tcp-lab-sim-cli -- --encda legacy_java/ENCDA.tcp --tui \
       --trace-out encda.json
   ```
5. **Headless grading**:
   ```bash
   cargo run -p tcp-lab-eval-host -- --scenario tests/scenarios/rdt2_basic.toml
   ```

## Loader Features & Built-ins

- Loader features are disabled by default to keep binaries lean. Add `--features "java"` or `"python"` etc. when you need a bridge.
- Built-in protocols: `--builtin-* rdt2` selects the stop-and-wait sender/receiver with timeouts and ACK handling. (This is also the default when you omit the flag.)

## Language SDKs

Starter projects with an RDT1 implementation and checksum utilities live in `sdk/`:

| Directory | Contents | How to run |
|-----------|----------|------------|
| `sdk/rust` | `tcp-lab-rust-sdk` crate with `rdt1` + checksum module. | `cargo run -p tcp-lab-rust-sdk --example rdt1_runner` |
| `sdk/python` | Python package (`tcp_lab` structs, `tcp_lab_sdk.rdt1`, checksum helper). | `uv pip install -e sdk/python` then `--python-sender tcp_lab_sdk.rdt1.Rdt1Sender` |
| `sdk/java` | Maven project exporting the JNI stubs and RDT1 classes. | `mvn package` then `--classpath ... --java-sender com.ouc.tcp.sdk.rdt1.Rdt1Sender` |
| `sdk/cpp` | Header-only helpers + CMake project building RDT1 sender/receiver libraries. | `cmake -B build && cmake --build build` then pass the `.so/.dll` via `--cpp-*` |

Each SDK ships a `checksum` helper so students can upgrade to RDT2 by adding checksum verification without rewriting boilerplate.

## Visualization Notes

- **Control bar** shows scenario name, current time, and pending events (`space` toggles pause, `s` steps once, `q` quits).
- **Link space-time diagram** paints sender/receiver timelines, channel events, and annotates drops/corruptions with seq/ack numbers.
- **Dashboard + Window panel** tracks deliveries, packet counts, and any reported metrics (`cwnd`, `ssthresh`) in the right half.
- **Link events list** retains the last ~100 events with color-coded severities. Use ↑/↓ to scroll.
- Use `--trace-out path.json` to persist the full `SimulationReport` for post-processing.

## Documentation

See `docs/ARCHITECTURE.md` for a deeper dive into the crate topology, loader feature flags, and how the simulator ties into the evaluation workflow.

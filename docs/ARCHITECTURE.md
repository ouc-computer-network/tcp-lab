# TCP Lab Architecture

This repository is now organized around the three stages of the workflow that surfaced in our design discussions: **loading student code**, **running headless evaluation**, and **visualizing simulations**. Each stage lives in its own crate so that language-specific concerns and UI logic are no longer mixed together.

## 1. `tcp-lab-abstract`

This library contains the language-agnostic pieces that every other crate depends on:

- The `TransportProtocol` and `SystemContext` traits that define the abstract functions students must implement.
- Packet/header definitions and flag helpers.
- Scenario descriptions (`TestScenario`, `TestAction`, `TestAssertion`) and the shared `SimConfig` struct.

Nothing in this crate knows how protocols are loaded or executed—it is pure interface and data.

## 2. `tcp-lab-loader`

The loader crate is the sole entry point for bringing external implementations into the Rust runtime. It exposes `ProtocolLoader` plus descriptors for each language. Every language binding sits behind its own cargo feature so we only compile the bridges that we need:

| Feature | Notes |
|---------|-------|
| `java`  | Wraps `tcp-lab-jni`, spins up a JVM with a configurable classpath, and exposes `ProtocolDescriptor::Java`. |
| `python`| Uses PyO3 plus a `PythonEnvironment` helper that asks the `uv` CLI for the target project's `sys.path`. Extra search paths (e.g., for ad‑hoc modules) can also be injected. |
| `cpp`   | Wraps the C/C++ ABI defined in `tcp-lab-ffi` so `.so/.dylib/.dll` loaders stay isolated. |

All of these features are **opt-in**; by default the loader only supports built-in Rust protocols. Enable languages explicitly via Cargo features, e.g. `cargo run -p tcp-lab-sim-cli --features "python" -- --python-sender …` or add `--features "java cpp"` for multiple bridges.

 Built-in Rust implementations (stop-and-wait “RDT2” sender/receiver) remain available through `ProtocolDescriptor::BuiltIn`, and native Rust implementations can be passed directly with `ProtocolDescriptor::Rust`.

## 3. `tcp-lab-eval-host`

This crate is the grade runner. It exposes a slim CLI (`cargo run -p tcp-lab-eval-host -- --scenario …`) that:

1. Configures the loader (classpath, uv project root, etc.).
2. Loads the sender/receiver according to CLI flags without knowing which language they came from.
3. Delegates to the simulator’s headless `scenario_runner` to obtain a pass/fail result.

There is purposely no TUI code here—this host just prints logs and exits with success/failure so it can be embedded into autograders.
Enable additional language bridges per need (`cargo run -p tcp-lab-eval-host --features "python" -- --python-sender …`).

## 4. `tcp-lab-simulator`

This crate houses all simulation logic:

- The deterministic event-based engine (`Simulator`, `NodeId`, `LinkEventSummary`).
- The `scenario_runner` module that replays `TestScenario` inputs and enforces assertions.
- An optional `tui` module (behind the `tui` feature) for interactive visualization/logging. Consumers that only need headless grading can omit that feature to keep dependencies small.
- A `trace` module that exposes `SimulationReport`, a serializable snapshot of a finished run (link events, metrics, deliveries) that downstream tools can archive or visualize later.
- An `encda` parser that understands the legacy encrypted `ENCDA.tcp` assets and converts them into chunks of application payloads to be scheduled in the simulator.

Future visualization binaries (e.g., playing back ENCDA.tcp or “simulate tragedy” traces) live here, consuming the same loader+abstract traits if they need to pull in student code.

## 5. `tcp-lab-sim-cli`

The developer-facing playground lives in this crate. It links the loader and the simulator’s TUI so you can run ad-hoc simulations, replay TOML scenarios, or decrypt historical traces (`ENCDA.tcp`) with visualization:

```
cargo run -p tcp-lab-sim-cli -- --tui --scenario tests/scenarios/gbn_timeout.toml \
    --python-sender examples.rdt3_sender.Rdt3Sender \
    --python-uv-project path/to/student/repo
```

Language bridges stay feature-gated here as well: use `cargo run -p tcp-lab-sim-cli --features "python"` for Python support, `--features "java"` for JVM, etc. Omit features to stick with Rust-only protocols.

Add `--trace-out trace.json` to export a JSON `SimulationReport` after every run—useful for offline animation or grading artifacts. To visualize the legacy encrypted ENCDA dataset, decrypt and queue it with `--encda legacy_java/ENCDA.tcp` (this flag is mutually exclusive with `--scenario`).

The CLI understands the same loader options as the eval host (Java classpath, uv project roots, built-in fallback protocols, etc.). Use it when you need to see packet timelines interactively; use the eval host when you just need pass/fail grades.

## Putting It Together

```
<student code in Java/Python/C++/Rust> --(feature-gated bridges)--> tcp-lab-loader
tcp-lab-loader --(TransportProtocol trait objects)--> tcp-lab-abstract

1) Evaluation path:
    loader + abstract scenario --> tcp-lab-eval-host --> tcp-lab-simulator::scenario_runner --> grade

2) Visualization path:
    loader/built-ins + abstract funcs --> tcp-lab-simulator (engine + optional TUI) --> logs/animation
```

Python developers should install [`uv`](https://github.com/astral-sh/uv) and point the loader at their project root (`--python-uv-project`). The loader automatically asks `uv run python` for the managed interpreter’s `sys.path`, ensuring embedded PyO3 imports match the virtual environment that `uv` controls.

## SDKs

Language-specific starter kits live in `sdk/`:

- `sdk/rust` – helper crate plus an RDT1 sender/receiver example and checksum utility.
- `sdk/python` – Python package containing the `tcp_lab.structs` module required by the loader, an RDT1 implementation, and checksum helper (installable via `uv pip install -e .`).
- `sdk/java` – Maven project that exposes the JNI stubs (`NativeBridge`, `SystemContextImpl`, etc.), checksum utils, and an RDT1 pair.
- `sdk/cpp` – CMake project with a header-only helper (`tcp_lab/sdk.hpp`), checksum helper, and ready-to-build RDT1 shared libraries.

Each SDK documents how to compile/run the reference RDT1 code and gives students a checksum helper so they can extend the template to RDT2.

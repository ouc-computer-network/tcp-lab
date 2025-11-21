# C++ SDK

This SDK ships two headers:

- `include/tcp_lab/sdk.hpp` – RAII helpers for calling into the Rust simulator plus the `TCP_LAB_REGISTER_PROTOCOL` macro that exports the required `create_protocol`/`protocol_*` symbols.
- `include/tcp_lab/checksum.hpp` – 16-bit Internet checksum helper for RDT2+.

It also contains a ready-to-build RDT1 sender/receiver pair (ideal channel).

## Build the templates

```
cd sdk/cpp
cmake -B build
cmake --build build
```

You will get `build/librdt1_sender.dylib` (or `.so`/`.dll`) and likewise for receiver.

Run them via the CLI:

```
cargo run -p tcp-lab-sim-cli -- \
  --cpp-sender sdk/cpp/build/librdt1_sender.dylib \
  --cpp-receiver sdk/cpp/build/librdt1_receiver.dylib \
  --scenario tests/scenarios/rdt2_basic.toml --tui
```

Port the template to RDT2 by editing the classes in `src/rdt1_*.cpp` and reusing the checksum helper.

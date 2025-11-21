# Python SDK

This package ships the data structures that the Rust loader expects (`tcp_lab.structs`) plus helpers for student implementations:

- `tcp_lab_sdk.protocol.BaseTransportProtocol` – base class with the required `init/on_packet/on_timer/on_app_data` hooks.
- `tcp_lab_sdk.checksum.internet_checksum` – 16-bit ones' complement helper for RDT2+.
- `tcp_lab_sdk.rdt1` – ready-to-use RDT1 sender/receiver built for a perfect channel.

## Installation (uv-managed virtualenv)

```
cd sdk/python
uv pip install -e .
```

Then point the simulator to this package:

```
cargo run -p tcp-lab-sim-cli -- --tui \
    --python-sender tcp_lab_sdk.rdt1.Rdt1Sender \
    --python-receiver tcp_lab_sdk.rdt1.Rdt1Receiver \
    --python-path sdk/python
```

When you move to RDT2, create your own module (e.g. `myteam.sender`) that inherits from `BaseTransportProtocol`, import `checksum.internet_checksum`, and pass the fully qualified name via `--python-sender`.

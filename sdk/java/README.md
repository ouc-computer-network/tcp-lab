# Java SDK

The `tcp-lab-java-sdk` module bundles the JNI-facing stubs (`Packet`, `TcpHeader`, `SystemContext`, `NativeBridge`) plus a reference RDT1 sender/receiver. Build it once and point the simulator at the resulting JAR.

## Build

```
cd sdk/java
mvn package
```

The jar lands in `target/tcp-lab-java-sdk-0.1.0.jar`.

## Run the reference implementation

```
cargo run -p tcp-lab-sim-cli -- \
  --classpath sdk/java/target/tcp-lab-java-sdk-0.1.0.jar \
  --java-sender com.ouc.tcp.sdk.rdt1.Rdt1Sender \
  --java-receiver com.ouc.tcp.sdk.rdt1.Rdt1Receiver \
  --scenario tests/scenarios/rdt2_basic.toml --tui
```

When you move to RDT2, create your own package (still depending on this SDK), subclass `TransportProtocol`, and reuse `util.Checksum.internetChecksum`.

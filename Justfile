# Justfile for TCP Lab

# Default target
default: build

# Build everything (Rust + Java)
build: build-java build-rust

# Build Rust crates
build-rust:
    cargo build

# Build Java SDK and Tests
build-java:
    mkdir -p sdk/java/classes
    javac -d sdk/java/classes sdk/java/src/com/ouc/tcp/sdk/*.java sdk/java/src/com/ouc/tcp/test/*.java sdk/java/src/com/ouc/tcp/impl/*.java sdk/java/src/com/ouc/tcp/legacy/*.java

# Run the simulator with pure Rust examples (TUI enabled)
run-rust: build-rust
    cargo run -- --tui

# Run the simulator with Java TestSender (TUI enabled)
run-java: build
    cargo run -- \
        --tui \
        --classpath sdk/java/classes \
        --java-sender com.ouc.tcp.test.TestSender \
        --java-receiver com.ouc.tcp.test.TestSender

# Tests
test-rdt20: build
    cargo run -- \
        --test-scenario tests/test_rdt20.toml \
        --classpath sdk/java/classes \
        --java-sender com.ouc.tcp.legacy.Rdt20Sender \
        --java-receiver com.ouc.tcp.legacy.Rdt20Receiver

test-rdt21: build
    cargo run -- \
        --test-scenario tests/test_rdt21.toml \
        --classpath sdk/java/classes \
        --java-sender com.ouc.tcp.legacy.Rdt21Sender \
        --java-receiver com.ouc.tcp.legacy.Rdt21Receiver

test-rdt22: build
    cargo run -- \
        --test-scenario tests/test_rdt22.toml \
        --classpath sdk/java/classes \
        --java-sender com.ouc.tcp.legacy.Rdt22Sender \
        --java-receiver com.ouc.tcp.legacy.Rdt22Receiver

test-rdt3: build
    cargo run -- \
        --test-scenario tests/test_rdt3.toml \
        --classpath sdk/java/classes \
        --java-sender com.ouc.tcp.impl.Rdt3Sender \
        --java-receiver com.ouc.tcp.impl.Rdt3Receiver

test-rdt3-ack: build
    cargo run -- \
        --test-scenario tests/test_rdt3_ack.toml \
        --classpath sdk/java/classes \
        --java-sender com.ouc.tcp.impl.Rdt3Sender \
        --java-receiver com.ouc.tcp.impl.Rdt3Receiver

test-reno: build
    cargo run -- \
        --test-scenario tests/test_reno.toml \
        --classpath sdk/java/classes \
        --java-sender com.ouc.tcp.impl.TcpRenoSender \
        --java-receiver com.ouc.tcp.impl.TcpRenoReceiver

test-tahoe: build
    cargo run -- \
        --test-scenario tests/test_tahoe.toml \
        --classpath sdk/java/classes \
        --java-sender com.ouc.tcp.impl.TcpTahoeSender \
        --java-receiver com.ouc.tcp.impl.TcpTahoeReceiver

test-gbn: build
    cargo run -- \
        --test-scenario tests/test_gbn.toml \
        --classpath sdk/java/classes \
        --java-sender com.ouc.tcp.impl.GbnSender \
        --java-receiver com.ouc.tcp.impl.GbnReceiver

test-sr: build
    cargo run -- \
        --test-scenario tests/test_sr.toml \
        --classpath sdk/java/classes \
        --java-sender com.ouc.tcp.impl.SrSender \
        --java-receiver com.ouc.tcp.impl.SrReceiver

run-rdt3: build
    cargo run -- \
        --tui \
        --classpath sdk/java/classes \
        --java-sender com.ouc.tcp.impl.Rdt3Sender \
        --java-receiver com.ouc.tcp.impl.Rdt3Receiver

run-gbn: build
    cargo run -- \
        --tui \
        --classpath sdk/java/classes \
        --java-sender com.ouc.tcp.impl.GbnSender \
        --java-receiver com.ouc.tcp.impl.GbnReceiver

run-sr: build
    cargo run -- \
        --tui \
        --classpath sdk/java/classes \
        --java-sender com.ouc.tcp.impl.SrSender \
        --java-receiver com.ouc.tcp.impl.SrReceiver

run-reno: build
    cargo run -- \
        --tui \
        --classpath sdk/java/classes \
        --java-sender com.ouc.tcp.impl.TcpRenoSender \
        --java-receiver com.ouc.tcp.impl.TcpRenoReceiver

run-tahoe: build
    cargo run -- \
        --tui \
        --classpath sdk/java/classes \
        --java-sender com.ouc.tcp.impl.TcpTahoeSender \
        --java-receiver com.ouc.tcp.impl.TcpTahoeReceiver

# Clean build artifacts
clean:
    cargo clean
    rm -rf sdk/java/classes

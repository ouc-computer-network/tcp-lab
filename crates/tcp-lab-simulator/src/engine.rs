use crate::trace::SimulationReport;
use rand::Rng;
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use tcp_lab_abstract::{Packet, SimConfig, flags};
use tcp_lab_abstract::{SystemContext, TransportProtocol};
use tracing::{debug, info};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeId {
    Sender,
    Receiver,
}

impl NodeId {
    pub fn peer(&self) -> Self {
        match self {
            NodeId::Sender => NodeId::Receiver,
            NodeId::Receiver => NodeId::Sender,
        }
    }
}

#[derive(Debug)]
pub enum EventType {
    PacketArrival {
        to: NodeId,
        packet: Packet,
    },
    TimerExpiry {
        node: NodeId,
        timer_id: u32,
        generation: u64,
    },
    AppSend {
        data: Vec<u8>,
    },
}

#[derive(Debug)]
struct Event {
    time: u64,
    event_type: EventType,
    id: u64, // Unique ID to differentiate events at same time
}

// Custom Ord for Min-Heap (smallest time pops first)
impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time && self.id == other.id
    }
}

impl Eq for Event {}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse comparison for time: smallest time is Greater in BinaryHeap
        other
            .time
            .cmp(&self.time)
            .then_with(|| other.id.cmp(&self.id))
    }
}

/// A compact textual summary of important link-layer events for visualization.
#[derive(Debug, Clone, Serialize)]
pub struct LinkEventSummary {
    pub time: u64,
    pub description: String,
}

/// Actions buffered during a student's function call
#[derive(Default)]
struct ActionBuffer {
    outgoing_packets: Vec<Packet>,
    timers_start: Vec<(u64, u32)>, // (delay, id)
    timers_cancel: Vec<u32>,
    logs: Vec<String>,
    delivered_data: Vec<Vec<u8>>,
    metrics: Vec<(String, f64)>,
}

/// Context implementation passed to the student
struct ScopedContext<'a> {
    buffer: &'a mut ActionBuffer,
    now: u64,
}

impl<'a> SystemContext for ScopedContext<'a> {
    fn send_packet(&mut self, packet: Packet) {
        self.buffer.outgoing_packets.push(packet);
    }

    fn start_timer(&mut self, delay_ms: u64, timer_id: u32) {
        self.buffer.timers_start.push((delay_ms, timer_id));
    }

    fn cancel_timer(&mut self, timer_id: u32) {
        self.buffer.timers_cancel.push(timer_id);
    }

    fn deliver_data(&mut self, data: &[u8]) {
        self.buffer.delivered_data.push(data.to_vec());
    }

    fn log(&mut self, message: &str) {
        self.buffer.logs.push(message.to_string());
    }

    fn now(&self) -> u64 {
        self.now
    }

    fn record_metric(&mut self, name: &str, value: f64) {
        self.buffer.metrics.push((name.to_string(), value));
    }
}

pub struct Simulator {
    time: u64,
    event_queue: BinaryHeap<Event>,
    event_id_counter: u64,

    config: SimConfig,
    rng: rand::rngs::StdRng,

    // We hold the two nodes directly
    // We use Box to allow different implementations
    pub sender: Box<dyn TransportProtocol>,
    pub receiver: Box<dyn TransportProtocol>,

    // Stats for Grader
    pub delivered_data: Vec<Vec<u8>>,
    pub sender_packet_count: u32,

    // Optional: record sender-side window size (e.g., cwnd) reported in header.window_size
    pub sender_window_sizes: Vec<u16>,

    /// Arbitrary time-series metrics recorded via `SystemContext::record_metric`
    /// Key: metric name (e.g., "ssthresh"), Value: Vec<(time_ms, value)>
    pub metrics: HashMap<String, Vec<(u64, f64)>>,

    // Deterministic fault injection: drop first packet from Sender with given seq numbers
    drop_sender_seq_once: Vec<u32>,
    // Deterministic fault injection: drop first ACK from Receiver with given ack numbers
    drop_receiver_ack_once: Vec<u32>,

    /// Timeline of link events (drops, corruptions, sends, deliveries) for TUI visualization.
    pub link_events: Vec<LinkEventSummary>,

    /// Timer generations to handle cancellation.
    /// Key: (node, timer_id), Value: generation counter
    timer_generations: HashMap<(NodeId, u32), u64>,
}

impl Simulator {
    pub fn new(
        config: SimConfig,
        sender: Box<dyn TransportProtocol>,
        receiver: Box<dyn TransportProtocol>,
    ) -> Self {
        use rand::SeedableRng;
        let rng = rand::rngs::StdRng::seed_from_u64(config.seed);

        Self {
            time: 0,
            event_queue: BinaryHeap::new(),
            event_id_counter: 0,
            config,
            rng,
            sender,
            receiver,
            delivered_data: Vec::new(),
            sender_packet_count: 0,
            sender_window_sizes: Vec::new(),
            metrics: HashMap::new(),
            drop_sender_seq_once: Vec::new(),
            drop_receiver_ack_once: Vec::new(),
            link_events: Vec::new(),
            timer_generations: HashMap::new(),
        }
    }

    /// Register a deterministic fault: drop the first packet sent by Sender whose seq equals `seq`.
    pub fn add_drop_sender_seq_once(&mut self, seq: u32) {
        self.drop_sender_seq_once.push(seq);
    }

    /// Register a deterministic fault: drop the first ACK sent by Receiver whose ack equals `ack`.
    pub fn add_drop_receiver_ack_once(&mut self, ack: u32) {
        self.drop_receiver_ack_once.push(ack);
    }

    /// Expose current simulation config (for TUI / diagnostics)
    pub fn config(&self) -> &SimConfig {
        &self.config
    }

    /// Return a slice of (time_ms, value) samples for a named metric, if present.
    pub fn metric_series(&self, name: &str) -> Option<&[(u64, f64)]> {
        self.metrics.get(name).map(|v| v.as_slice())
    }

    fn push_event(&mut self, time: u64, event_type: EventType) {
        self.event_queue.push(Event {
            time,
            event_type,
            id: self.event_id_counter,
        });
        self.event_id_counter += 1;
    }

    pub fn schedule_app_send(&mut self, time: u64, data: Vec<u8>) {
        self.push_event(time, EventType::AppSend { data });
    }

    pub fn init(&mut self) {
        // Init phase
        {
            let mut buffer = ActionBuffer::default();
            let mut ctx = ScopedContext {
                buffer: &mut buffer,
                now: self.time,
            };
            self.sender.init(&mut ctx);
            self.process_actions(NodeId::Sender, buffer);
        }
        {
            let mut buffer = ActionBuffer::default();
            let mut ctx = ScopedContext {
                buffer: &mut buffer,
                now: self.time,
            };
            self.receiver.init(&mut ctx);
            self.process_actions(NodeId::Receiver, buffer);
        }
    }

    pub fn peek_next_event_time(&self) -> Option<u64> {
        self.event_queue.peek().map(|e| e.time)
    }

    pub fn current_time(&self) -> u64 {
        self.time
    }

    pub fn remaining_events(&self) -> usize {
        self.event_queue.len()
    }

    /// Process the next event. Returns true if an event was processed, false if queue is empty.
    pub fn step(&mut self) -> bool {
        let event = match self.event_queue.pop() {
            Some(e) => e,
            None => return false,
        };

        self.time = event.time;
        debug!("Processing event at {}: {:?}", self.time, event.event_type);

        match event.event_type {
            EventType::PacketArrival { to, packet } => {
                let mut buffer = ActionBuffer::default();
                {
                    let mut ctx = ScopedContext {
                        buffer: &mut buffer,
                        now: self.time,
                    };
                    match to {
                        NodeId::Sender => self.sender.on_packet(&mut ctx, packet),
                        NodeId::Receiver => self.receiver.on_packet(&mut ctx, packet),
                    }
                }
                self.process_actions(to, buffer);
            }
            EventType::TimerExpiry {
                node,
                timer_id,
                generation,
            } => {
                // Check if this timer event is still valid by comparing generations
                let key = (node, timer_id);
                if let Some(&current_generation) = self.timer_generations.get(&key) {
                    if current_generation != generation {
                        // This timer has been cancelled, skip the callback
                        debug!("Skipping cancelled timer event for timer_id={}", timer_id);
                        return true; // Event processed (by being ignored)
                    }
                } else {
                    // No record of this timer, it might be from a previous simulation run
                    // or an orphaned event. Skip it for safety.
                    debug!("Skipping orphaned timer event for timer_id={}", timer_id);
                    return true; // Event processed (by being ignored)
                }

                let mut buffer = ActionBuffer::default();
                {
                    let mut ctx = ScopedContext {
                        buffer: &mut buffer,
                        now: self.time,
                    };
                    match node {
                        NodeId::Sender => self.sender.on_timer(&mut ctx, timer_id),
                        NodeId::Receiver => self.receiver.on_timer(&mut ctx, timer_id),
                    }
                }
                self.process_actions(node, buffer);
            }
            EventType::AppSend { data } => {
                let mut buffer = ActionBuffer::default();
                {
                    let mut ctx = ScopedContext {
                        buffer: &mut buffer,
                        now: self.time,
                    };
                    self.sender.on_app_data(&mut ctx, &data);
                }
                self.process_actions(NodeId::Sender, buffer);
            }
        }
        true
    }

    /// Produce a serializable snapshot of the current simulation state.
    pub fn export_report(&self) -> SimulationReport {
        SimulationReport {
            config: self.config.clone(),
            duration_ms: self.time,
            delivered_data: self.delivered_data.clone(),
            sender_packet_count: self.sender_packet_count,
            sender_window_sizes: self.sender_window_sizes.clone(),
            metrics: self.metrics.clone(),
            link_events: self.link_events.clone(),
        }
    }

    pub fn run_until_complete(&mut self) {
        self.init();
        while self.step() {}
    }

    fn process_actions(&mut self, source_node: NodeId, buffer: ActionBuffer) {
        // First, fold metrics into simulator-wide store
        for (name, value) in buffer.metrics {
            self.metrics
                .entry(name)
                .or_default()
                .push((self.time, value));
        }

        for log in buffer.logs {
            info!("[{:?}] {}", source_node, log);
        }

        for data in buffer.delivered_data {
            info!("[{:?}] DELIVERED DATA: {} bytes", source_node, data.len());
            self.link_events.push(LinkEventSummary {
                time: self.time,
                description: format!(
                    "[{:?}] DELIVERED {} bytes to application",
                    source_node,
                    data.len()
                ),
            });
            self.delivered_data.push(data);
        }

        // Handle timer cancellations by incrementing the generation counter
        for timer_id in buffer.timers_cancel {
            let key = (source_node, timer_id);
            // Increment the generation to invalidate existing timer events
            let generation = self.timer_generations.entry(key).or_insert(0);
            *generation += 1;
        }

        for (delay, id) in buffer.timers_start {
            let key = (source_node, id);
            let generation = *self.timer_generations.entry(key).or_insert(0);
            self.push_event(
                self.time + delay,
                EventType::TimerExpiry {
                    node: source_node,
                    timer_id: id,
                    generation,
                },
            );
        }

        // Packet transmission logic (Channel)
        for mut packet in buffer.outgoing_packets {
            if source_node == NodeId::Sender {
                self.sender_packet_count += 1;

                // 记录 sender 发包时报告的 window size（如果非零）
                if packet.header.window_size > 0 {
                    self.sender_window_sizes.push(packet.header.window_size);
                }

                // Deterministic SR/GBN tests: optionally drop first packet with given seq
                if let Some(pos) = self
                    .drop_sender_seq_once
                    .iter()
                    .position(|s| *s == packet.header.seq_num)
                {
                    self.link_events.push(LinkEventSummary {
                        time: self.time,
                        description: format!(
                            "[Sender->Receiver] DROP (deterministic seq) seq={}",
                            packet.header.seq_num
                        ),
                    });
                    debug!(
                        "Deterministically dropping sender packet with seq={}",
                        packet.header.seq_num
                    );
                    self.drop_sender_seq_once.remove(pos);
                    continue;
                }
            }

            if source_node == NodeId::Receiver {
                // Deterministic tests: optionally drop first ACK with given ack number
                if packet.header.flags & flags::ACK != 0
                    && let Some(pos) = self
                        .drop_receiver_ack_once
                        .iter()
                        .position(|a| *a == packet.header.ack_num)
                {
                    self.link_events.push(LinkEventSummary {
                        time: self.time,
                        description: format!(
                            "[Receiver->Sender] DROP (deterministic ack) ack={}",
                            packet.header.ack_num
                        ),
                    });
                    debug!(
                        "Deterministically dropping receiver ACK with ack={}",
                        packet.header.ack_num
                    );
                    self.drop_receiver_ack_once.remove(pos);
                    continue;
                }
            }

            // 1. Check Loss
            if self.rng.random::<f64>() < self.config.loss_rate {
                self.link_events.push(LinkEventSummary {
                    time: self.time,
                    description: format!(
                        "[{:?}->{:?}] DROP (random loss) seq={} ack={}",
                        source_node,
                        source_node.peer(),
                        packet.header.seq_num,
                        packet.header.ack_num
                    ),
                });
                debug!("Packet lost in channel");
                continue;
            }

            // 2. Check Corruption
            if self.rng.random::<f64>() < self.config.corrupt_rate {
                self.link_events.push(LinkEventSummary {
                    time: self.time,
                    description: format!(
                        "[{:?}->{:?}] CORRUPT seq={} ack={}",
                        source_node,
                        source_node.peer(),
                        packet.header.seq_num,
                        packet.header.ack_num
                    ),
                });
                debug!("Packet corrupted in channel");
                // Simple corruption: flip the checksum to make it invalid
                packet.header.checksum = !packet.header.checksum;
            }

            // 3. Calculate Latency
            let latency = self
                .rng
                .random_range(self.config.min_latency..=self.config.max_latency);
            let arrival_time = self.time + latency;

            // 4. Target Node
            let target_node = source_node.peer();

            self.link_events.push(LinkEventSummary {
                time: self.time,
                description: format!(
                    "[{:?}->{:?}] SEND seq={} ack={} (latency={}ms)",
                    source_node, target_node, packet.header.seq_num, packet.header.ack_num, latency
                ),
            });

            self.push_event(
                arrival_time,
                EventType::PacketArrival {
                    to: target_node,
                    packet,
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Simulator;
    use tcp_lab_abstract::{Packet, SimConfig, SystemContext, TransportProtocol};

    struct TestProtocol {
        timer_fired: bool,
        timer_cancelled: bool,
    }

    impl TestProtocol {
        fn new() -> Self {
            Self {
                timer_fired: false,
                timer_cancelled: false,
            }
        }
    }

    impl TransportProtocol for TestProtocol {
        fn init(&mut self, _ctx: &mut dyn SystemContext) {
            // Start a timer that will fire in 10ms
            _ctx.start_timer(10, 0);
            // Schedule a dummy event to cancel the timer after it has been started
            _ctx.start_timer(5, 1); // This timer will trigger the cancellation
        }

        fn on_packet(&mut self, _ctx: &mut dyn SystemContext, _packet: Packet) {
            // Not used in this test
        }

        fn on_timer(&mut self, _ctx: &mut dyn SystemContext, timer_id: u32) {
            match timer_id {
                0 => {
                    // This should NOT be called if the timer was successfully cancelled
                    self.timer_fired = true;
                }
                1 => {
                    // Cancel the first timer
                    _ctx.cancel_timer(0);
                    self.timer_cancelled = true;
                }
                _ => {}
            }
        }

        fn on_app_data(&mut self, _ctx: &mut dyn SystemContext, _data: &[u8]) {
            // Not used in this test
        }
    }

    #[test]
    fn test_cancel_timer() {
        let config = SimConfig::default();
        let sender = Box::new(TestProtocol::new());
        let receiver = Box::new(TestProtocol::new());

        let mut simulator = Simulator::new(config, sender, receiver);

        // Run the simulation
        simulator.run_until_complete();

        // Extract the protocols back to check their state
        // We need to use unsafe code here because we can't move out of Box<dyn Trait>
        // This is just for testing purposes
        let sender_ptr = simulator.sender.as_ref() as *const dyn TransportProtocol;
        let sender_state = unsafe {
            let concrete = sender_ptr as *const TestProtocol;
            &*concrete
        };

        // The timer should have been cancelled but not fired
        assert!(
            sender_state.timer_cancelled,
            "Timer should have been cancelled"
        );
        assert!(
            !sender_state.timer_fired,
            "Cancelled timer should not have fired"
        );
    }
}

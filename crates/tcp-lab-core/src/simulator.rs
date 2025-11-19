use std::cmp::Ordering;
use std::collections::BinaryHeap;
use rand::Rng;
use tracing::{debug, info};
use crate::interface::{SystemContext, TransportProtocol};
use crate::packet::{Packet, flags};

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
        other.time.cmp(&self.time)
            .then_with(|| other.id.cmp(&self.id))
    }
}

#[derive(Debug, Clone)]
pub struct SimConfig {
    pub loss_rate: f64,
    pub corrupt_rate: f64,
    pub min_latency: u64,
    pub max_latency: u64,
    pub seed: u64,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            loss_rate: 0.0,
            corrupt_rate: 0.0,
            min_latency: 10,
            max_latency: 100,
            seed: 0,
        }
    }
}

/// Actions buffered during a student's function call
struct ActionBuffer {
    outgoing_packets: Vec<Packet>,
    timers_start: Vec<(u64, u32)>, // (delay, id)
    timers_cancel: Vec<u32>,
    logs: Vec<String>,
    delivered_data: Vec<Vec<u8>>,
}

impl Default for ActionBuffer {
    fn default() -> Self {
        Self {
            outgoing_packets: Vec::new(),
            timers_start: Vec::new(),
            timers_cancel: Vec::new(),
            logs: Vec::new(),
            delivered_data: Vec::new(),
        }
    }
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

    // Deterministic fault injection: drop first packet from Sender with given seq numbers
    drop_sender_seq_once: Vec<u32>,
    // Deterministic fault injection: drop first ACK from Receiver with given ack numbers
    drop_receiver_ack_once: Vec<u32>,
}

impl Simulator {
    pub fn new(config: SimConfig, sender: Box<dyn TransportProtocol>, receiver: Box<dyn TransportProtocol>) -> Self {
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
            drop_sender_seq_once: Vec::new(),
            drop_receiver_ack_once: Vec::new(),
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
            let mut ctx = ScopedContext { buffer: &mut buffer, now: self.time };
            self.sender.init(&mut ctx);
            self.process_actions(NodeId::Sender, buffer);
        }
        {
            let mut buffer = ActionBuffer::default();
            let mut ctx = ScopedContext { buffer: &mut buffer, now: self.time };
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
                    let mut ctx = ScopedContext { buffer: &mut buffer, now: self.time };
                    match to {
                        NodeId::Sender => self.sender.on_packet(&mut ctx, packet),
                        NodeId::Receiver => self.receiver.on_packet(&mut ctx, packet),
                    }
                }
                self.process_actions(to, buffer);
            }
            EventType::TimerExpiry { node, timer_id } => {
                let mut buffer = ActionBuffer::default();
                {
                    let mut ctx = ScopedContext { buffer: &mut buffer, now: self.time };
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
                    let mut ctx = ScopedContext { buffer: &mut buffer, now: self.time };
                    self.sender.send(&mut ctx, &data);
                }
                self.process_actions(NodeId::Sender, buffer);
            }
        }
        true
    }

    pub fn run_until_complete(&mut self) {
        self.init();
        while self.step() {}
    }

    fn process_actions(&mut self, source_node: NodeId, buffer: ActionBuffer) {
        for log in buffer.logs {
            info!("[{:?}] {}", source_node, log);
        }

        for data in buffer.delivered_data {
             info!("[{:?}] DELIVERED DATA: {} bytes", source_node, data.len());
             self.delivered_data.push(data);
        }

        for (delay, id) in buffer.timers_start {
            self.push_event(self.time + delay, EventType::TimerExpiry {
                node: source_node,
                timer_id: id,
            });
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
                    debug!("Deterministically dropping sender packet with seq={}", packet.header.seq_num);
                    self.drop_sender_seq_once.remove(pos);
                    continue;
                }
            }

            if source_node == NodeId::Receiver {
                // Deterministic tests: optionally drop first ACK with given ack number
                if packet.header.flags & flags::ACK != 0 {
                    if let Some(pos) = self
                        .drop_receiver_ack_once
                        .iter()
                        .position(|a| *a == packet.header.ack_num)
                    {
                        debug!("Deterministically dropping receiver ACK with ack={}", packet.header.ack_num);
                        self.drop_receiver_ack_once.remove(pos);
                        continue;
                    }
                }
            }

            // 1. Check Loss
            if self.rng.random::<f64>() < self.config.loss_rate {
                debug!("Packet lost in channel");
                continue;
            }

            // 2. Check Corruption
            if self.rng.random::<f64>() < self.config.corrupt_rate {
                debug!("Packet corrupted in channel");
                // Simple corruption: flip the checksum to make it invalid
                packet.header.checksum = !packet.header.checksum;
            }

            // 3. Calculate Latency
            let latency = self.rng.random_range(self.config.min_latency..=self.config.max_latency);
            let arrival_time = self.time + latency;

            // 4. Target Node
            let target_node = source_node.peer();

            self.push_event(arrival_time, EventType::PacketArrival {
                to: target_node,
                packet,
            });
        }
    }
}

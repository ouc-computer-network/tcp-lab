use std::collections::VecDeque;
use tcp_lab_abstract::{Packet, SystemContext, TransportProtocol, flags};

const DATA_TIMER: u32 = 1;
const DATA_TIMEOUT_MS: u64 = 1000;

fn checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut chunks = data.chunks_exact(2);
    for chunk in &mut chunks {
        let word = u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
        sum = sum.wrapping_add(word);
    }
    if let Some(&byte) = chunks.remainder().first() {
        sum = sum.wrapping_add((byte as u32) << 8);
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

#[derive(Default)]
pub struct Rdt2Sender {
    next_seq: u32,
    waiting_ack: bool,
    pending: VecDeque<Vec<u8>>,
    last_packet: Option<Packet>,
}

impl Rdt2Sender {
    fn try_send(&mut self, ctx: &mut dyn SystemContext) {
        if self.waiting_ack {
            return;
        }
        if let Some(payload) = self.pending.pop_front() {
            let mut packet = Packet::new_simple(self.next_seq, 0, 0, payload);
            packet.header.checksum = checksum(&packet.payload);
            ctx.log(&format!(
                "RDT2 send seq={} ({} bytes)",
                self.next_seq,
                packet.len()
            ));
            ctx.send_packet(packet.clone());
            ctx.start_timer(DATA_TIMEOUT_MS, DATA_TIMER);
            self.last_packet = Some(packet);
            self.waiting_ack = true;
        }
    }

    fn handle_ack(&mut self, ctx: &mut dyn SystemContext, ack: u32) {
        if !self.waiting_ack || ack != self.next_seq {
            return;
        }
        ctx.log(&format!("RDT2 received ACK for seq {}", ack));
        ctx.cancel_timer(DATA_TIMER);
        self.waiting_ack = false;
        self.next_seq ^= 1;
        self.try_send(ctx);
    }
}

impl TransportProtocol for Rdt2Sender {
    fn init(&mut self, ctx: &mut dyn SystemContext) {
        ctx.log("RDT2 sender ready");
    }

    fn on_packet(&mut self, ctx: &mut dyn SystemContext, packet: Packet) {
        if packet.header.flags & flags::ACK != 0 {
            self.handle_ack(ctx, packet.header.ack_num);
        }
    }

    fn on_timer(&mut self, ctx: &mut dyn SystemContext, timer_id: u32) {
        if timer_id != DATA_TIMER || !self.waiting_ack {
            return;
        }
        if let Some(packet) = self.last_packet.clone() {
            ctx.log(&format!(
                "RDT2 timeout, retransmitting seq {}",
                packet.header.seq_num
            ));
            ctx.send_packet(packet.clone());
            ctx.start_timer(DATA_TIMEOUT_MS, DATA_TIMER);
            self.last_packet = Some(packet);
        }
    }

    fn on_app_data(&mut self, ctx: &mut dyn SystemContext, data: &[u8]) {
        self.pending.push_back(data.to_vec());
        self.try_send(ctx);
    }
}

#[derive(Default)]
pub struct Rdt2Receiver {
    expected_seq: u32,
    last_acked: u32,
}

impl Rdt2Receiver {
    fn send_ack(&mut self, ctx: &mut dyn SystemContext, seq: u32) {
        let ack = Packet::new_ack(seq, seq, 0);
        ctx.log(&format!("RDT2 send ACK for seq {}", seq));
        ctx.send_packet(ack);
        self.last_acked = seq;
    }
}

impl TransportProtocol for Rdt2Receiver {
    fn init(&mut self, ctx: &mut dyn SystemContext) {
        ctx.log("RDT2 receiver ready");
        self.last_acked = self.expected_seq ^ 1;
    }

    fn on_packet(&mut self, ctx: &mut dyn SystemContext, packet: Packet) {
        let expected_checksum = checksum(&packet.payload);
        if expected_checksum != packet.header.checksum {
            ctx.log(&format!(
                "RDT2 checksum mismatch for seq {} (expected {:04X}, got {:04X})",
                packet.header.seq_num, expected_checksum, packet.header.checksum
            ));
            self.send_ack(ctx, self.last_acked);
            return;
        }
        if packet.header.seq_num == self.expected_seq {
            ctx.log(&format!(
                "RDT2 received seq {} ({} bytes)",
                packet.header.seq_num,
                packet.len()
            ));
            ctx.deliver_data(&packet.payload);
            self.send_ack(ctx, packet.header.seq_num);
            self.expected_seq ^= 1;
        } else {
            ctx.log(&format!(
                "RDT2 unexpected seq {} (expect {}), re-ACK {}",
                packet.header.seq_num, self.expected_seq, self.last_acked
            ));
            self.send_ack(ctx, self.last_acked);
        }
    }

    fn on_timer(&mut self, _ctx: &mut dyn SystemContext, _timer_id: u32) {}

    fn on_app_data(&mut self, _ctx: &mut dyn SystemContext, _data: &[u8]) {}
}

pub fn rdt2_sender() -> Box<dyn TransportProtocol> {
    Box::new(Rdt2Sender::default())
}

pub fn rdt2_receiver() -> Box<dyn TransportProtocol> {
    Box::new(Rdt2Receiver::default())
}

pub fn default_sender() -> Box<dyn TransportProtocol> {
    rdt2_sender()
}

pub fn default_receiver() -> Box<dyn TransportProtocol> {
    rdt2_receiver()
}

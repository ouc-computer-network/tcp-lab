use tcp_lab_abstract::{Packet, SystemContext, TransportProtocol};

/// Minimal RDT1 sender (assumes a perfect channel, no checksum/ACK).
#[derive(Default)]
pub struct Rdt1Sender;

impl TransportProtocol for Rdt1Sender {
    fn init(&mut self, ctx: &mut dyn SystemContext) {
        ctx.log("RDT1 sender ready (ideal channel)");
    }

    fn on_packet(&mut self, _ctx: &mut dyn SystemContext, _packet: Packet) {
        // Nothing to do; RDT1 ignores all inbound messages.
    }

    fn on_timer(&mut self, _ctx: &mut dyn SystemContext, _timer_id: u32) {
        // No timers needed for an ideal channel.
    }

    fn on_app_data(&mut self, ctx: &mut dyn SystemContext, data: &[u8]) {
        let packet = Packet::new_simple(0, 0, 0, data.to_vec());
        ctx.log(&format!(
            "RDT1 sender pushing {} bytes to channel",
            packet.payload.len()
        ));
        ctx.send_packet(packet);
    }
}

/// Minimal RDT1 receiver (immediately delivers application data).
#[derive(Default)]
pub struct Rdt1Receiver;

impl TransportProtocol for Rdt1Receiver {
    fn init(&mut self, ctx: &mut dyn SystemContext) {
        ctx.log("RDT1 receiver ready (ideal channel)");
    }

    fn on_packet(&mut self, ctx: &mut dyn SystemContext, packet: Packet) {
        ctx.log(&format!(
            "RDT1 receiver delivering {} bytes",
            packet.payload.len()
        ));
        ctx.deliver_data(&packet.payload);
    }

    fn on_timer(&mut self, _ctx: &mut dyn SystemContext, _timer_id: u32) {}

    fn on_app_data(&mut self, _ctx: &mut dyn SystemContext, _data: &[u8]) {}
}

pub fn sender() -> Box<dyn TransportProtocol> {
    Box::new(Rdt1Sender::default())
}

pub fn receiver() -> Box<dyn TransportProtocol> {
    Box::new(Rdt1Receiver::default())
}

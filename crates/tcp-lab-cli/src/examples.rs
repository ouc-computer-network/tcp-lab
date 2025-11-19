use tcp_lab_core::{Packet, SystemContext, TransportProtocol};

#[derive(Default)]
pub struct SimpleSender;

impl TransportProtocol for SimpleSender {
    fn init(&mut self, ctx: &mut dyn SystemContext) {
        ctx.log("Sender initialized");
    }

    fn on_packet(&mut self, _ctx: &mut dyn SystemContext, _packet: Packet) {
        // Ignore ACKs for now
    }

    fn on_timer(&mut self, _ctx: &mut dyn SystemContext, _timer_id: u32) {
        // No timers yet
    }

    fn send(&mut self, ctx: &mut dyn SystemContext, data: &[u8]) {
        ctx.log(&format!("Sending data: {:?}", data));
        let packet = Packet::new_simple(0, 0, 0, data.to_vec());
        ctx.send_packet(packet);
    }
}

#[derive(Default)]
pub struct SimpleReceiver;

impl TransportProtocol for SimpleReceiver {
    fn init(&mut self, ctx: &mut dyn SystemContext) {
        ctx.log("Receiver initialized");
    }

    fn on_packet(&mut self, ctx: &mut dyn SystemContext, packet: Packet) {
        ctx.log(&format!("Received packet with {} bytes", packet.len()));
        ctx.deliver_data(&packet.payload);

        // Send ACK
        let ack = Packet::new_ack(0, packet.header.seq_num, 100);
        ctx.send_packet(ack);
    }

    fn on_timer(&mut self, _ctx: &mut dyn SystemContext, _timer_id: u32) {}

    fn send(&mut self, _ctx: &mut dyn SystemContext, _data: &[u8]) {}
}

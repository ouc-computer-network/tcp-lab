use tcp_lab_core::{Packet, SystemContext, TransportProtocol};

pub struct Rdt3Sender {
    seq_num: u32,
}

impl Rdt3Sender {
    pub fn new() -> Self {
        Self { seq_num: 0 }
    }
}

impl TransportProtocol for Rdt3Sender {
    fn init(&mut self, ctx: &mut dyn SystemContext) {
        ctx.log("Rdt3Sender initialized");
    }

    fn on_packet(&mut self, ctx: &mut dyn SystemContext, packet: Packet) {
        if packet.header.ack_num == self.seq_num {
            ctx.log(&format!("Received ACK for seq {}", self.seq_num));
            self.seq_num += 1;
        }
    }

    fn on_timer(&mut self, _ctx: &mut dyn SystemContext, _timer_id: u32) {
        // Retransmit logic would go here
    }

    fn on_app_data(&mut self, ctx: &mut dyn SystemContext, data: &[u8]) {
        ctx.log(&format!("Sending data: {:?}", data));
        let packet = Packet::new_simple(self.seq_num, 0, 0, data.to_vec());
        ctx.send_packet(packet);
    }
}
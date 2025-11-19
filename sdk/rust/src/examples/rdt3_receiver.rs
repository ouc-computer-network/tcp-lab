use crate::{Packet, SystemContext, TcpHeader, TransportProtocol, flags};

pub struct Rdt3Receiver {
    expected_seq: u32,
}

impl Rdt3Receiver {
    pub fn new() -> Self {
        Self { expected_seq: 0 }
    }

    fn send_ack(&self, ctx: &mut dyn SystemContext, ack_num: u32) {
        let mut h = TcpHeader::default();
        h.ack_num = ack_num;
        h.flags = flags::ACK;
        let packet = Packet::new(h, vec![]);
        ctx.send_packet(packet);
    }
}

impl TransportProtocol for Rdt3Receiver {
    fn init(&mut self, _ctx: &mut dyn SystemContext) {
        self.expected_seq = 0;
    }

    fn on_app_data(&mut self, _ctx: &mut dyn SystemContext, _data: &[u8]) {
        // Receiver doesn't send app data
    }

    fn on_packet(&mut self, ctx: &mut dyn SystemContext, packet: Packet) {
        let h = &packet.header;

        if h.seq_num == self.expected_seq {
            ctx.log(&format!("Received correct packet {}", self.expected_seq));
            ctx.deliver_data(&packet.payload);
            self.send_ack(ctx, self.expected_seq);
            self.expected_seq = 1 - self.expected_seq;
        } else {
            ctx.log(&format!(
                "Duplicate/Out-of-order packet {}, expected {}",
                h.seq_num, self.expected_seq
            ));
            // Re-send ACK for the LAST correctly received packet (1 - expected_seq)
            // Note: sequences are 0 and 1, so 1-0=1, 1-1=0.
            self.send_ack(ctx, 1 - self.expected_seq);
        }
    }

    fn on_timer(&mut self, _ctx: &mut dyn SystemContext, _timer_id: u32) {
        // Receiver has no timers
    }
}

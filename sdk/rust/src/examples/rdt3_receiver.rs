use tcp_lab_core::{Packet, SystemContext, TransportProtocol};

pub struct Rdt3Receiver {
    expected_seq: u32,
}

impl Rdt3Receiver {
    pub fn new() -> Self {
        Self { expected_seq: 0 }
    }
}

impl TransportProtocol for Rdt3Receiver {
    fn init(&mut self, ctx: &mut dyn SystemContext) {
        ctx.log("Rdt3Receiver initialized");
    }

    fn on_packet(&mut self, ctx: &mut dyn SystemContext, packet: Packet) {
        if packet.header.seq_num == self.expected_seq {
            ctx.log(&format!("Received expected packet seq {}", packet.header.seq_num));
            ctx.deliver_data(&packet.payload);
            
            let ack = Packet::new_ack(0, self.expected_seq, 100);
            ctx.send_packet(ack);
            
            self.expected_seq += 1;
        } else {
            ctx.log(&format!("Received unexpected packet seq {}, expected {}", packet.header.seq_num, self.expected_seq));
            // Resend ACK for last correctly received packet (if any)
            if self.expected_seq > 0 {
                let ack = Packet::new_ack(0, self.expected_seq - 1, 100);
                ctx.send_packet(ack);
            }
        }
    }

    fn on_timer(&mut self, _ctx: &mut dyn SystemContext, _timer_id: u32) {}

    fn on_app_data(&mut self, _ctx: &mut dyn SystemContext, _data: &[u8]) {}
}
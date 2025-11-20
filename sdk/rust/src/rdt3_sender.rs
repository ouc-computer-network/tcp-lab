use tcp_lab_core::{flags, Packet, SystemContext, TcpHeader, TransportProtocol};

pub struct Rdt3Sender {
    next_seq: u32,
    is_waiting: bool,
    current_packet: Option<Packet>,
}

impl Rdt3Sender {
    pub fn new() -> Self {
        Self {
            next_seq: 0,
            is_waiting: false,
            current_packet: None,
        }
    }

    fn checksum(seq: u32, data: &[u8]) -> u16 {
        let mut sum = seq;
        for &b in data {
            sum += b as u32;
        }
        (sum & 0xFFFF) as u16
    }

    fn is_corrupted(h: &TcpHeader, payload: &[u8]) -> bool {
        let expected = Self::checksum(h.seq_num, payload);
        expected != h.checksum
    }
}

impl TransportProtocol for Rdt3Sender {
    fn init(&mut self, _ctx: &mut dyn SystemContext) {
        self.next_seq = 0;
        self.is_waiting = false;
        self.current_packet = None;
    }

    fn on_app_data(&mut self, ctx: &mut dyn SystemContext, data: &[u8]) {
        if self.is_waiting {
            ctx.log("RDT3 Sender Busy: Dropping application data");
            return;
        }

        let mut h = TcpHeader::default();
        h.seq_num = self.next_seq;
        h.checksum = Self::checksum(h.seq_num, data);

        let packet = Packet::new(h, data.to_vec());
        self.current_packet = Some(packet.clone());

        ctx.send_packet(packet);
        ctx.start_timer(3000, self.next_seq);
        self.is_waiting = true;
    }

    fn on_packet(&mut self, ctx: &mut dyn SystemContext, packet: Packet) {
        if !self.is_waiting {
            return;
        }

        let h = &packet.header;
        let corrupted = Self::is_corrupted(h, &packet.payload);

        if corrupted || ((h.flags & flags::ACK != 0) && h.ack_num != self.next_seq) {
            ctx.log(&format!(
                "Corrupted or Duplicate ACK. Retransmitting {}",
                self.next_seq
            ));
            ctx.cancel_timer(self.next_seq);
            if let Some(pkt) = &self.current_packet {
                ctx.send_packet(pkt.clone());
                ctx.start_timer(3000, self.next_seq);
            }
            return;
        }

        if (h.flags & flags::ACK != 0) && h.ack_num == self.next_seq {
            ctx.log(&format!("Received ACK {}", self.next_seq));
            ctx.cancel_timer(self.next_seq);
            self.is_waiting = false;
            self.next_seq = 1 - self.next_seq;
        }
    }

    fn on_timer(&mut self, ctx: &mut dyn SystemContext, timer_id: u32) {
        if self.is_waiting && timer_id == self.next_seq {
            ctx.log(&format!("Timeout! Retransmitting seq {}", self.next_seq));
            if let Some(pkt) = &self.current_packet {
                ctx.send_packet(pkt.clone());
                ctx.start_timer(3000, self.next_seq);
            }
        }
    }
}

from typing import Optional
from tcp_lab import TransportProtocol, SystemContext, Packet, TcpHeader, TcpFlags

def checksum(seq: int, data: bytes) -> int:
    s = seq
    for b in data:
        s += b
    return s & 0xFFFF

def is_corrupted(h: TcpHeader, payload: bytes) -> bool:
    return checksum(h.seq_num, payload) != h.checksum

class Rdt3Sender(TransportProtocol):
    def __init__(self):
        self.next_seq = 0
        self.is_waiting = False
        self.current_packet: Optional[Packet] = None

    def init(self, ctx: SystemContext) -> None:
        self.next_seq = 0
        self.is_waiting = False
        self.current_packet = None

    def on_app_data(self, ctx: SystemContext, data: bytes) -> None:
        if self.is_waiting:
            ctx.log("RDT3 Sender Busy: Dropping application data")
            return

        h = TcpHeader(seq_num=self.next_seq)
        h.checksum = checksum(h.seq_num, data)
        
        self.current_packet = Packet(h, data)
        
        ctx.send_packet(self.current_packet)
        ctx.start_timer(3000, self.next_seq)
        self.is_waiting = True

    def on_packet(self, ctx: SystemContext, packet: Packet) -> None:
        if not self.is_waiting:
            return

        h = packet.header
        corrupted = is_corrupted(h, packet.payload)

        if corrupted or (h.is_ack and h.ack_num != self.next_seq):
            ctx.log(f"Corrupted or Duplicate ACK. Retransmitting {self.next_seq}")
            ctx.cancel_timer(self.next_seq)
            if self.current_packet:
                ctx.send_packet(self.current_packet)
                ctx.start_timer(3000, self.next_seq)
            return

        if h.is_ack and h.ack_num == self.next_seq:
            ctx.log(f"Received ACK {self.next_seq}")
            ctx.cancel_timer(self.next_seq)
            self.is_waiting = False
            self.next_seq = 1 - self.next_seq

    def on_timer(self, ctx: SystemContext, timer_id: int) -> None:
        if self.is_waiting and timer_id == self.next_seq:
            ctx.log(f"Timeout! Retransmitting seq {self.next_seq}")
            if self.current_packet:
                ctx.send_packet(self.current_packet)
                ctx.start_timer(3000, self.next_seq)


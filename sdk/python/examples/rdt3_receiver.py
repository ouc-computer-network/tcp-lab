from tcp_lab import TransportProtocol, SystemContext, Packet, TcpHeader, TcpFlags

class Rdt3Receiver(TransportProtocol):
    def __init__(self):
        self.expected_seq = 0

    def init(self, ctx: SystemContext) -> None:
        self.expected_seq = 0

    def on_app_data(self, ctx: SystemContext, data: bytes) -> None:
        pass

    def on_packet(self, ctx: SystemContext, packet: Packet) -> None:
        h = packet.header
        
        if h.seq_num == self.expected_seq:
            ctx.log(f"Received correct packet {self.expected_seq}")
            ctx.deliver_data(packet.payload)
            self.send_ack(ctx, self.expected_seq)
            self.expected_seq = 1 - self.expected_seq
        else:
            ctx.log(f"Duplicate/Out-of-order packet {h.seq_num}, expected {self.expected_seq}")
            self.send_ack(ctx, 1 - self.expected_seq)

    def on_timer(self, ctx: SystemContext, timer_id: int) -> None:
        pass

    def send_ack(self, ctx: SystemContext, ack_num: int) -> None:
        h = TcpHeader(ack_num=ack_num, flags=TcpFlags.ACK)
        ctx.send_packet(Packet(h, b""))


"""Reference RDT1 transport protocol for a perfect channel."""

from __future__ import annotations

from tcp_lab.structs import Packet, TcpHeader

from .protocol import BaseTransportProtocol, SystemContext


class Rdt1Sender(BaseTransportProtocol):
    def init(self, ctx: SystemContext) -> None:
        ctx.log("Python RDT1 sender ready")

    def on_packet(self, ctx: SystemContext, packet: Packet) -> None:
        ctx.log(f"RDT1 sender ignoring inbound packet seq={packet.header.seq_num}")

    def on_app_data(self, ctx: SystemContext, data: bytes) -> None:
        packet = Packet(TcpHeader(seq_num=0, ack_num=0), data)
        ctx.log(f"RDT1 sender pushing {len(data)} bytes")
        ctx.send_packet(packet)


class Rdt1Receiver(BaseTransportProtocol):
    def init(self, ctx: SystemContext) -> None:
        ctx.log("Python RDT1 receiver ready")

    def on_packet(self, ctx: SystemContext, packet: Packet) -> None:
        ctx.log(f"RDT1 receiver delivering {len(packet.payload)} bytes")
        ctx.deliver_data(packet.payload)

    def on_app_data(self, ctx: SystemContext, data: bytes) -> None:
        ctx.log(f"RDT1 receiver ignoring outbound app data of len {len(data)}")

from __future__ import annotations

from dataclasses import dataclass, field
from typing import ByteString


@dataclass
class TcpHeader:
    seq_num: int = 0
    ack_num: int = 0
    flags: int = 0
    window_size: int = 0
    checksum: int = 0

    def setSeqNum(self, value: int) -> None:  # JNI expects camelCase setters
        self.seq_num = value

    def setAckNum(self, value: int) -> None:
        self.ack_num = value

    def setFlags(self, value: int) -> None:
        self.flags = value

    def setWindowSize(self, value: int) -> None:
        self.window_size = value

    def setChecksum(self, value: int) -> None:
        self.checksum = value


@dataclass
class Packet:
    header: TcpHeader = field(default_factory=TcpHeader)
    payload: bytes = field(default_factory=bytes)

    def __init__(self, header: TcpHeader, payload: ByteString):
        self.header = header
        self.payload = bytes(payload)

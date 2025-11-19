from dataclasses import dataclass
from enum import IntFlag

class TcpFlags(IntFlag):
    FIN = 0x01
    SYN = 0x02
    RST = 0x04
    PSH = 0x08
    ACK = 0x10
    URG = 0x20

@dataclass
class TcpHeader:
    seq_num: int = 0
    ack_num: int = 0
    flags: int = 0
    window_size: int = 0
    checksum: int = 0
    urgent_ptr: int = 0
    
    @property
    def is_syn(self) -> bool:
        return bool(self.flags & TcpFlags.SYN)
        
    @property
    def is_ack(self) -> bool:
        return bool(self.flags & TcpFlags.ACK)
        
    @property
    def is_fin(self) -> bool:
        return bool(self.flags & TcpFlags.FIN)

@dataclass
class Packet:
    header: TcpHeader
    payload: bytes

    def __init__(self, header: TcpHeader, payload: bytes):
        self.header = header
        self.payload = payload


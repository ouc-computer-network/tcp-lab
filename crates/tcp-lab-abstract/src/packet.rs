use serde::{Deserialize, Serialize};

/// TCP Header flags
pub mod flags {
    pub const FIN: u8 = 0x01;
    pub const SYN: u8 = 0x02;
    pub const RST: u8 = 0x04;
    pub const PSH: u8 = 0x08;
    pub const ACK: u8 = 0x10;
    pub const URG: u8 = 0x20;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub struct TcpHeader {
    /// Source Port (Optional in our simple 1-to-1 sim, but kept for realism)
    pub src_port: u16,
    /// Destination Port
    pub dst_port: u16,
    /// Sequence Number
    pub seq_num: u32,
    /// Acknowledgment Number
    pub ack_num: u32,
    /// Data Offset (Header Length) & Reserved & Flags
    /// We simplify and just store flags directly for the lab
    pub flags: u8,
    /// Window Size
    pub window_size: u16,
    /// Checksum (Simulator will handle verification, but students might need to calc it in RDT phase)
    pub checksum: u16,
    /// Urgent Pointer
    pub urgent_ptr: u16,
}


impl TcpHeader {
    pub fn new(seq: u32, ack: u32, flags: u8, wnd: u16) -> Self {
        Self {
            seq_num: seq,
            ack_num: ack,
            flags,
            window_size: wnd,
            ..Default::default()
        }
    }

    pub fn is_syn(&self) -> bool {
        self.flags & flags::SYN != 0
    }
    pub fn is_ack(&self) -> bool {
        self.flags & flags::ACK != 0
    }
    pub fn is_fin(&self) -> bool {
        self.flags & flags::FIN != 0
    }
    pub fn is_rst(&self) -> bool {
        self.flags & flags::RST != 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    pub header: TcpHeader,
    pub payload: Vec<u8>,
}

impl Packet {
    pub fn new(header: TcpHeader, payload: Vec<u8>) -> Self {
        Self { header, payload }
    }

    pub fn new_simple(seq: u32, ack: u32, flags: u8, payload: Vec<u8>) -> Self {
        Self {
            header: TcpHeader::new(seq, ack, flags, 0),
            payload,
        }
    }

    /// Create a pure ACK packet
    pub fn new_ack(seq: u32, ack: u32, window: u16) -> Self {
        Self {
            header: TcpHeader::new(seq, ack, flags::ACK, window),
            payload: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.payload.len() // Simplified: only payload length matters for some metrics
    }
}

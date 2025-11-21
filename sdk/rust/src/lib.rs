//! Rust SDK for TCP Lab student implementations.
//! Provides checksum helpers and a reference RDT1 sender/receiver.

pub mod checksum;
pub mod rdt1;

pub use tcp_lab_abstract::{Packet, SystemContext, TransportProtocol};

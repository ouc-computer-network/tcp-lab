pub mod config;
pub mod interface;
pub mod packet;
pub mod scenario;

pub use interface::{SystemContext, TransportProtocol};
pub use packet::{Packet, TcpHeader};
// Re-export flags module from packet so users can access TcpHeader::Flags
pub use packet::flags;

pub use config::SimConfig;
pub use scenario::{SimConfigOverride, TestAction, TestAssertion, TestScenario};

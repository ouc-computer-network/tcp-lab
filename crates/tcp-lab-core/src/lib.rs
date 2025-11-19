pub mod grader;
pub mod interface;
pub mod packet;
pub mod simulator; // Add this

pub use interface::{SystemContext, TransportProtocol};
pub use packet::{Packet, TcpHeader};
// Re-export flags module from packet so users can access TcpHeader::Flags
pub use packet::flags;

pub use grader::{TestAction, TestAssertion, TestScenario};
pub use simulator::{NodeId, SimConfig, Simulator};

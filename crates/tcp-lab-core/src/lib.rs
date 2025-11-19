pub mod interface;
pub mod packet;
pub mod simulator;
pub mod grader; // Add this

pub use interface::{SystemContext, TransportProtocol};
pub use packet::{Packet, TcpHeader};
// Re-export flags module from packet so users can access TcpHeader::Flags
pub use packet::flags; 

pub use simulator::{Simulator, SimConfig, NodeId};
pub use grader::{TestScenario, TestAction, TestAssertion};

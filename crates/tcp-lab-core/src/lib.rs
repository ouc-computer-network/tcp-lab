pub mod interface;
pub mod packet;
pub mod simulator;
pub mod grader; // Add this

pub use interface::{SystemContext, TransportProtocol};
pub use packet::{Packet, TcpHeader};
pub use simulator::{Simulator, SimConfig, NodeId};
pub use grader::{TestScenario, TestAction, TestAssertion};

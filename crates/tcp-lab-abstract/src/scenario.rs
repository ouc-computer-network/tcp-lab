use crate::config::SimConfig;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct TestScenario {
    pub name: String,
    pub description: String,
    pub config: SimConfigOverride,
    pub actions: Vec<TestAction>,
    pub assertions: Vec<TestAssertion>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SimConfigOverride {
    pub loss_rate: Option<f64>,
    pub corrupt_rate: Option<f64>,
    pub min_latency: Option<u64>,
    pub max_latency: Option<u64>,
    pub seed: Option<u64>,
}

impl SimConfigOverride {
    pub fn apply_to(&self, config: &mut SimConfig) {
        if let Some(v) = self.loss_rate {
            config.loss_rate = v;
        }
        if let Some(v) = self.corrupt_rate {
            config.corrupt_rate = v;
        }
        if let Some(v) = self.min_latency {
            config.min_latency = v;
        }
        if let Some(v) = self.max_latency {
            config.max_latency = v;
        }
        if let Some(v) = self.seed {
            config.seed = v;
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TestAction {
    /// Application sends data at a specific time
    AppSend { time: u64, data: String },
    /// Deterministically drop the first packet sent by Sender with given seq number
    DropNextFromSenderSeq { seq: u32 },
    /// Deterministically drop the first ACK sent by Receiver with given ack number
    DropNextFromReceiverAck { ack: u32 },
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TestAssertion {
    /// Assert that specific data was delivered to the application layer
    DataDelivered { data: String },
    /// Assert that the total number of packets sent by Sender is within range
    SenderPacketCount { min: u32, max: Option<u32> },
    /// Assert that the maximum window size (as reported in header.window_size by sender) is within range
    SenderWindowMax { min: u16, max: Option<u16> },
    /// Assert that the window size eventually drops from at least `from_at_least` down to at most `to_at_most`
    SenderWindowDrop { from_at_least: u16, to_at_most: u16 },
    /// Assert that simulation finishes within time
    MaxDuration { ms: u64 },
}

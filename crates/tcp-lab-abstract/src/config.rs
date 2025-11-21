use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimConfig {
    pub loss_rate: f64,
    pub corrupt_rate: f64,
    pub min_latency: u64,
    pub max_latency: u64,
    pub seed: u64,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            loss_rate: 0.0,
            corrupt_rate: 0.0,
            min_latency: 10,
            max_latency: 100,
            seed: 0,
        }
    }
}

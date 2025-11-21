use serde::Serialize;
use std::collections::HashMap;
use tcp_lab_abstract::SimConfig;

use crate::engine::LinkEventSummary;

#[derive(Debug, Clone, Serialize)]
pub struct SimulationReport {
    pub config: SimConfig,
    pub duration_ms: u64,
    pub delivered_data: Vec<Vec<u8>>,
    pub sender_packet_count: u32,
    pub sender_window_sizes: Vec<u16>,
    pub metrics: HashMap<String, Vec<(u64, f64)>>,
    pub link_events: Vec<LinkEventSummary>,
}

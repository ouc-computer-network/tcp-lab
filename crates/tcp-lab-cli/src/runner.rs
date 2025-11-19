use anyhow::{Context, anyhow};
use std::fs;
use tcp_lab_core::{
    SimConfig, Simulator, TestAction, TestAssertion, TestScenario, TransportProtocol,
};
use tracing::info;

pub fn run_scenario(
    scenario_path: &str,
    sender: Box<dyn TransportProtocol>,
    receiver: Box<dyn TransportProtocol>,
) -> anyhow::Result<()> {
    let content = fs::read_to_string(scenario_path).context("Failed to read scenario file")?;
    let scenario: TestScenario = toml::from_str(&content).context("Failed to parse scenario")?;

    info!("Running Scenario: {}", scenario.name);
    info!("Description: {}", scenario.description);

    let mut config = SimConfig::default();
    scenario.config.apply_to(&mut config);

    let mut sim = Simulator::new(config, sender, receiver);

    // Configure actions (App sends, deterministic faults, etc.)
    for action in &scenario.actions {
        match action {
            TestAction::AppSend { time, data } => {
                sim.schedule_app_send(*time, data.as_bytes().to_vec());
            }
            TestAction::DropNextFromSenderSeq { seq } => {
                sim.add_drop_sender_seq_once(*seq);
            }
            TestAction::DropNextFromReceiverAck { ack } => {
                sim.add_drop_receiver_ack_once(*ack);
            }
        }
    }

    // Call init after we've configured the simulator
    sim.init();

    // Max duration check
    let max_duration = scenario
        .assertions
        .iter()
        .find_map(|a| {
            if let TestAssertion::MaxDuration { ms } = a {
                Some(*ms)
            } else {
                None
            }
        })
        .unwrap_or(10000); // Default 10s

    // Run loop
    while sim.step() {
        if sim.current_time() > max_duration {
            return Err(anyhow!("Test timed out after {} ms", max_duration));
        }
    }

    // Final assertions
    for assertion in &scenario.assertions {
        match assertion {
            TestAssertion::DataDelivered { data } => {
                let found = sim.delivered_data.iter().any(|d| d == data.as_bytes());
                if !found {
                    return Err(anyhow!(
                        "Assertion Failed: Data {:?} was not delivered",
                        data
                    ));
                }
            }
            TestAssertion::SenderPacketCount { min, max } => {
                if sim.sender_packet_count < *min {
                    return Err(anyhow!(
                        "Assertion Failed: Sender sent {} packets, expected min {}",
                        sim.sender_packet_count,
                        min
                    ));
                }
                if let Some(max) = max {
                    if sim.sender_packet_count > *max {
                        return Err(anyhow!(
                            "Assertion Failed: Sender sent {} packets, expected max {}",
                            sim.sender_packet_count,
                            max
                        ));
                    }
                }
            }
            TestAssertion::SenderWindowMax { min, max } => {
                let max_win = sim.sender_window_sizes.iter().copied().max().unwrap_or(0);
                if max_win < *min {
                    return Err(anyhow!(
                        "Assertion Failed: Sender window max {} < expected min {}",
                        max_win,
                        min
                    ));
                }
                if let Some(m) = max {
                    if max_win > *m {
                        return Err(anyhow!(
                            "Assertion Failed: Sender window max {} > expected max {}",
                            max_win,
                            m
                        ));
                    }
                }
            }
            TestAssertion::SenderWindowDrop {
                from_at_least,
                to_at_most,
            } => {
                let mut seen_high = false;
                let mut seen_drop = false;
                for w in &sim.sender_window_sizes {
                    if !seen_high && *w >= *from_at_least {
                        seen_high = true;
                    } else if seen_high && *w <= *to_at_most {
                        seen_drop = true;
                        break;
                    }
                }
                if !seen_high || !seen_drop {
                    return Err(anyhow!(
                        "Assertion Failed: Sender window did not drop from >= {} down to <= {}",
                        from_at_least,
                        to_at_most
                    ));
                }
            }
            TestAssertion::MaxDuration { .. } => {} // Already checked
        }
    }

    info!("Test Scenario Passed!");
    Ok(())
}

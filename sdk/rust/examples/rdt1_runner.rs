use tcp_lab_abstract::SimConfig;
use tcp_lab_rust_sdk::rdt1::{receiver, sender};
use tcp_lab_simulator::{SimulationReport, Simulator};

fn main() {
    let mut sim = Simulator::new(SimConfig::default(), sender(), receiver());
    sim.schedule_app_send(0, b"Hello from RDT1".to_vec());
    sim.schedule_app_send(10, b"This channel is perfect".to_vec());
    sim.run_until_complete();

    let report: SimulationReport = sim.export_report();
    println!("Delivered {} messages", report.delivered_data.len());
}

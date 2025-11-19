use crate::packet::Packet;

/// The capability provided by the simulator to the student's protocol.
/// Students call these methods to interact with the network and application layer.
pub trait SystemContext {
    /// Send a packet to the network (unreliable channel).
    fn send_packet(&mut self, packet: Packet);

    /// Start a timer.
    /// `timer_id` is a user-defined ID to identify this timer (e.g. matching a sequence number).
    /// `delay_ms` is the duration in milliseconds.
    /// Note: If a timer with the same ID already exists, behavior depends on implementation (usually overwrite or dual).
    /// Recommendation: Use unique IDs or cancel before start.
    fn start_timer(&mut self, delay_ms: u64, timer_id: u32);

    /// Cancel a running timer.
    fn cancel_timer(&mut self, timer_id: u32);

    /// Deliver data to the Application Layer (e.g. when a sequence is complete and valid).
    fn deliver_data(&mut self, data: &[u8]);

    /// Log a message to the simulator's debug output.
    fn log(&mut self, message: &str);
    
    /// Get current simulation time in ms
    fn now(&self) -> u64;

    /// Record a numeric metric for visualization / grading (e.g., cwnd, ssthresh).
    /// Implementations may aggregate these for later inspection in the TUI or grader.
    fn record_metric(&mut self, _name: &str, _value: f64) {
        // Default no-op so non-visual environments don't need to care.
    }
}

/// The interface that students must implement.
pub trait TransportProtocol {
    /// Called when the simulation starts.
    fn init(&mut self, _ctx: &mut dyn SystemContext) {}

    /// Called when a packet arrives from the network.
    fn on_packet(&mut self, ctx: &mut dyn SystemContext, packet: Packet);

    /// Called when a timer expires.
    fn on_timer(&mut self, ctx: &mut dyn SystemContext, timer_id: u32);

    /// Called when the Application Layer wants to send data reliably.
    /// The protocol should encapsulate this data into packets and send them.
    fn on_app_data(&mut self, ctx: &mut dyn SystemContext, data: &[u8]);
}


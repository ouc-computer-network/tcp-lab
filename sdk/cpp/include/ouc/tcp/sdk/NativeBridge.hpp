// C++ SDK: NativeBridge declarations
// These C-ABI functions are implemented by the Rust host (tcp-lab-ffi),
// and used internally by SystemContextImpl.
//
// As a student, you normally do *not* call these directly; instead you use
// the high-level SystemContext interface.

#pragma once

#include <cstddef>
#include <cstdint>

namespace ouc::tcp::sdk::native {

extern "C" {

/// Send a packet into the simulator.
void tcp_lab_send_packet(std::uint32_t seq,
                         std::uint32_t ack,
                         std::uint8_t flags,
                         std::uint16_t window,
                         std::uint16_t checksum,
                         const std::uint8_t* payload,
                         std::size_t payload_len);

/// Start a timer.
void tcp_lab_start_timer(std::uint64_t delay_ms, std::int32_t timer_id);

/// Cancel a timer.
void tcp_lab_cancel_timer(std::int32_t timer_id);

/// Deliver data to the application.
void tcp_lab_deliver_data(const std::uint8_t* data, std::size_t len);

/// Log a debug message.
void tcp_lab_log(const char* msg);

/// Current simulation time (ms).
std::uint64_t tcp_lab_now();

/// Record a numeric metric (for TUI/grader).
void tcp_lab_record_metric(const char* name, double value);

} // extern "C"

} // namespace ouc::tcp::sdk::native



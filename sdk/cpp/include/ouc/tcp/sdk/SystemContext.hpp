// C++ SDK: SystemContext interface
// Students use this interface to interact with the simulator:
//   - send packets
//   - start/cancel timers
//   - deliver data to the application
//   - log messages

#pragma once

#include <cstdint>
#include <string>
#include <vector>

#include "Packet.hpp"

namespace ouc::tcp::sdk {

class SystemContext {
public:
    virtual ~SystemContext() = default;

    /// Send a packet into the unreliable channel.
    virtual void sendPacket(const Packet& packet) = 0;

    /// Start a timer that will trigger onTimer(timerId) after delayMs.
    virtual void startTimer(std::uint64_t delayMs, int timerId) = 0;

    /// Cancel a previously started timer.
    virtual void cancelTimer(int timerId) = 0;

    /// Deliver data up to the application layer.
    virtual void deliverData(const std::vector<std::uint8_t>& data) = 0;

    /// Log a debug message (visible in the TUI / logs).
    virtual void log(const std::string& message) = 0;

    /// Current simulation time in ms.
    virtual std::uint64_t now() const = 0;

    /// Record a numeric metric for visualization / grading (e.g. cwnd, ssthresh).
    /// Default implementation is a no-op so student code can call it safely.
    virtual void recordMetric(const std::string& /*name*/, double /*value*/) {
        // no-op by default
    }
};

} // namespace ouc::tcp::sdk



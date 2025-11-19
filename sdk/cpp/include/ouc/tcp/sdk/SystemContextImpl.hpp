// C++ SDK: SystemContext implementation that delegates to the Rust host
// via the C ABI functions declared in NativeBridge.hpp.
//
// This is analogous to the Java SystemContextImpl + NativeBridge pair.

#pragma once

#include <string>
#include <vector>
#include <cstdint>

#include "SystemContext.hpp"
#include "NativeBridge.hpp"

namespace ouc::tcp::sdk {

class SystemContextImpl : public SystemContext {
public:
    void sendPacket(const Packet& packet) override {
        const TcpHeader& h = packet.header;
        const std::vector<std::uint8_t>& p = packet.payload;
        native::tcp_lab_send_packet(
            h.seqNum,
            h.ackNum,
            h.flags,
            h.windowSize,
            h.checksum,
            p.data(),
            p.size()
        );
    }

    void startTimer(std::uint64_t delayMs, int timerId) override {
        native::tcp_lab_start_timer(delayMs, static_cast<std::int32_t>(timerId));
    }

    void cancelTimer(int timerId) override {
        native::tcp_lab_cancel_timer(static_cast<std::int32_t>(timerId));
    }

    void deliverData(const std::vector<std::uint8_t>& data) override {
        native::tcp_lab_deliver_data(data.data(), data.size());
    }

    void log(const std::string& message) override {
        native::tcp_lab_log(message.c_str());
    }

    std::uint64_t now() const override {
        return native::tcp_lab_now();
    }

    void recordMetric(const std::string& name, double value) override {
        native::tcp_lab_record_metric(name.c_str(), value);
    }
};

} // namespace ouc::tcp::sdk



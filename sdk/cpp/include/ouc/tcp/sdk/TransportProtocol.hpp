// C++ SDK: TransportProtocol base class
// Students should inherit from this class to implement a protocol (RDT, GBN, SR, TCP Reno/Tahoe, etc.).

#pragma once

#include <vector>
#include <cstdint>

#include "SystemContext.hpp"
#include "Packet.hpp"

namespace ouc::tcp::sdk {

class TransportProtocol {
public:
    virtual ~TransportProtocol() = default;

    /// Called once when the simulation starts.
    virtual void init(SystemContext& /*ctx*/) {}

    /// Called when a packet arrives from the network.
    virtual void onPacket(SystemContext& ctx, const Packet& packet) = 0;

    /// Called when a timer expires.
    virtual void onTimer(SystemContext& ctx, int timerId) = 0;

    /// Called when the application wants to send new data reliably.
    virtual void onAppData(SystemContext& ctx, const std::vector<std::uint8_t>& data) = 0;
};

} // namespace ouc::tcp::sdk



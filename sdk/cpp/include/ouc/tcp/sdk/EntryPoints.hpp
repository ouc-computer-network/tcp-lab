// Helper macros to define the C entrypoints expected by the Rust loader
// for a C++ protocol implementation.
//
// This hides all the boilerplate for:
//   - create_sender / destroy_sender
//   - sender_init
//   - sender_on_app_data
//   - sender_on_packet
//   - sender_on_timer
//
// Usage (in your C++ protocol library):
//
//   #include <ouc/tcp/sdk/TransportProtocol.hpp>
//   #include <ouc/tcp/sdk/EntryPoints.hpp>
//
//   using namespace ouc::tcp::sdk;
//
//   class MySender : public TransportProtocol {
//       ... implement init/onAppData/onPacket/onTimer ...
//   };
//
//   TCP_LAB_DEFINE_PROTOCOL_ENTRYPOINTS(MySender);
//

#pragma once

#include <cstdint>
#include <vector>

#include "Export.hpp"
#include "TransportProtocol.hpp"
#include "SystemContextImpl.hpp"
#include "Packet.hpp"
#include "TcpHeader.hpp"

namespace ouc::tcp::sdk {

/// Define the standard sender entrypoints for a given protocol class.
/// The generated functions have C linkage and are exported from the DLL/so:
///   - create_sender()
///   - destroy_sender(TransportProtocol*)
///   - sender_init(TransportProtocol*)
///   - sender_on_app_data(TransportProtocol*, const uint8_t*, size_t)
///   - sender_on_packet(TransportProtocol*, ...)
///   - sender_on_timer(TransportProtocol*, int)
#define TCP_LAB_DEFINE_PROTOCOL_ENTRYPOINTS(ProtocolClass)                              \
    TCP_LAB_EXPORT ::ouc::tcp::sdk::TransportProtocol* create_sender() {               \
        return new ProtocolClass();                                                    \
    }                                                                                  \
                                                                                       \
    TCP_LAB_EXPORT void destroy_sender(::ouc::tcp::sdk::TransportProtocol* p) {        \
        delete p;                                                                      \
    }                                                                                  \
                                                                                       \
    TCP_LAB_EXPORT void sender_init(::ouc::tcp::sdk::TransportProtocol* p) {           \
        ::ouc::tcp::sdk::SystemContextImpl ctx;                                        \
        p->init(ctx);                                                                  \
    }                                                                                  \
                                                                                       \
    TCP_LAB_EXPORT void sender_on_app_data(                                            \
        ::ouc::tcp::sdk::TransportProtocol* p,                                         \
        const std::uint8_t* data,                                                      \
        std::size_t len) {                                                             \
        ::ouc::tcp::sdk::SystemContextImpl ctx;                                        \
        std::vector<std::uint8_t> v;                                                   \
        if (data && len > 0) {                                                         \
            v.assign(data, data + len);                                                \
        }                                                                              \
        p->onAppData(ctx, v);                                                          \
    }                                                                                  \
                                                                                       \
    TCP_LAB_EXPORT void sender_on_packet(                                              \
        ::ouc::tcp::sdk::TransportProtocol* p,                                         \
        std::uint32_t seq,                                                             \
        std::uint32_t ack,                                                             \
        std::uint8_t flags,                                                            \
        std::uint16_t window,                                                          \
        std::uint16_t checksum,                                                        \
        const std::uint8_t* payload,                                                   \
        std::size_t len) {                                                             \
        ::ouc::tcp::sdk::SystemContextImpl ctx;                                        \
        ::ouc::tcp::sdk::TcpHeader h;                                                  \
        h.seqNum = seq;                                                                \
        h.ackNum = ack;                                                                \
        h.flags = flags;                                                               \
        h.windowSize = window;                                                         \
        h.checksum = checksum;                                                         \
        std::vector<std::uint8_t> v;                                                   \
        if (payload && len > 0) {                                                      \
            v.assign(payload, payload + len);                                          \
        }                                                                              \
        ::ouc::tcp::sdk::Packet pkt{h, v};                                             \
        p->onPacket(ctx, pkt);                                                         \
    }                                                                                  \
                                                                                       \
    TCP_LAB_EXPORT void sender_on_timer(                                               \
        ::ouc::tcp::sdk::TransportProtocol* p,                                         \
        int timerId) {                                                                 \
        ::ouc::tcp::sdk::SystemContextImpl ctx;                                        \
        p->onTimer(ctx, timerId);                                                      \
    }

} // namespace ouc::tcp::sdk



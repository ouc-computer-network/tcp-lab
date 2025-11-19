// C++ SDK: TcpHeader definition
// This mirrors the Rust/Java TcpHeader used by the simulator.

#pragma once

#include <cstdint>

namespace ouc::tcp::sdk {

struct TcpHeader {
    struct Flags {
        static constexpr std::uint8_t FIN = 0x01;
        static constexpr std::uint8_t SYN = 0x02;
        static constexpr std::uint8_t RST = 0x04;
        static constexpr std::uint8_t PSH = 0x08;
        static constexpr std::uint8_t ACK = 0x10;
        static constexpr std::uint8_t URG = 0x20;
    };

    std::uint16_t srcPort{0};
    std::uint16_t dstPort{0};
    std::uint32_t seqNum{0};
    std::uint32_t ackNum{0};
    std::uint8_t  flags{0};
    std::uint16_t windowSize{0};
    std::uint16_t checksum{0};
    std::uint16_t urgentPtr{0};

    bool isSyn() const noexcept { return (flags & Flags::SYN) != 0; }
    bool isAck() const noexcept { return (flags & Flags::ACK) != 0; }
    bool isFin() const noexcept { return (flags & Flags::FIN) != 0; }
};

} // namespace ouc::tcp::sdk



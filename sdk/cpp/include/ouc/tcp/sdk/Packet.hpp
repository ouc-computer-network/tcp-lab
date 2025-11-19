// C++ SDK: Packet definition
// A simple value type containing a TcpHeader and payload bytes.

#pragma once

#include <vector>
#include <cstdint>

#include "TcpHeader.hpp"

namespace ouc::tcp::sdk {

struct Packet {
    TcpHeader header;
    std::vector<std::uint8_t> payload;

    Packet() = default;

    Packet(const TcpHeader& h, const std::vector<std::uint8_t>& data)
        : header(h), payload(data) {}

    Packet(TcpHeader&& h, std::vector<std::uint8_t>&& data)
        : header(std::move(h)), payload(std::move(data)) {}
};

} // namespace ouc::tcp::sdk



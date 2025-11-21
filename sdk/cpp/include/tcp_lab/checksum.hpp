#pragma once

#include <cstdint>
#include <cstddef>

namespace tcp_lab::sdk {

inline uint16_t internet_checksum(const uint8_t* data, size_t len) {
    uint32_t sum = 0;
    const uint8_t* ptr = data;

    while (len > 1) {
        sum += (ptr[0] << 8) | ptr[1];
        ptr += 2;
        len -= 2;

        if (sum & 0xFFFF0000) {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
    }

    if (len > 0) {
        sum += (ptr[0] << 8);
        if (sum & 0xFFFF0000) {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
    }

    return static_cast<uint16_t>(~sum);
}

} // namespace tcp_lab::sdk

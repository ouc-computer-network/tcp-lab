#pragma once

#include <cstdint>
#include <string>
#include <vector>

extern "C" {
void tcp_lab_send_packet(uint32_t seq, uint32_t ack, uint8_t flags, uint16_t window, uint16_t checksum,
                         const uint8_t* payload, size_t payload_len);
void tcp_lab_start_timer(uint64_t delay_ms, int32_t timer_id);
void tcp_lab_cancel_timer(int32_t timer_id);
void tcp_lab_deliver_data(const uint8_t* data, size_t len);
void tcp_lab_log(const char* msg);
uint64_t tcp_lab_now();
void tcp_lab_record_metric(const char* name, double value);
}

namespace tcp_lab::sdk {

struct TcpHeader {
    uint32_t seq_num = 0;
    uint32_t ack_num = 0;
    uint8_t flags = 0;
    uint16_t window_size = 0;
    uint16_t checksum = 0;
};

inline void send_packet(const TcpHeader& header, const std::vector<uint8_t>& payload) {
    tcp_lab_send_packet(header.seq_num, header.ack_num, header.flags, header.window_size, header.checksum,
                        payload.data(), payload.size());
}

inline void deliver_data(const std::vector<uint8_t>& data) {
    tcp_lab_deliver_data(data.data(), data.size());
}

inline void start_timer(uint64_t delay_ms, int timer_id) {
    tcp_lab_start_timer(delay_ms, timer_id);
}

inline void cancel_timer(int timer_id) {
    tcp_lab_cancel_timer(timer_id);
}

inline void log(const std::string& message) {
    tcp_lab_log(message.c_str());
}

inline uint64_t now() {
    return tcp_lab_now();
}

inline void record_metric(const std::string& name, double value) {
    tcp_lab_record_metric(name.c_str(), value);
}

class Protocol {
  public:
    virtual ~Protocol() = default;
    virtual void init() {}
    virtual void on_packet(const TcpHeader& header, const std::vector<uint8_t>& payload) {}
    virtual void on_timer(int timer_id) {}
    virtual void on_app_data(const std::vector<uint8_t>& data) {}
};

#define TCP_LAB_REGISTER_PROTOCOL(CLASS)                                                               \
    extern "C" ::tcp_lab::sdk::Protocol* create_protocol() { return new CLASS(); }                      \
    extern "C" void destroy_protocol(::tcp_lab::sdk::Protocol* ptr) { delete ptr; }                     \
    extern "C" void protocol_init(::tcp_lab::sdk::Protocol* ptr) { ptr->init(); }                       \
    extern "C" void protocol_on_app_data(::tcp_lab::sdk::Protocol* ptr, const uint8_t* data, size_t len)\
    {                                                                                                   \
        std::vector<uint8_t> buffer(data, data + len);                                                  \
        ptr->on_app_data(buffer);                                                                       \
    }                                                                                                   \
    extern "C" void protocol_on_packet(::tcp_lab::sdk::Protocol* ptr, uint32_t seq, uint32_t ack,       \
                                       uint8_t flags, uint16_t window, uint16_t checksum,               \
                                       const uint8_t* payload, size_t len)                              \
    {                                                                                                   \
        TcpHeader header{};                                                                             \
        header.seq_num = seq;                                                                           \
        header.ack_num = ack;                                                                           \
        header.flags = flags;                                                                           \
        header.window_size = window;                                                                    \
        header.checksum = checksum;                                                                     \
        std::vector<uint8_t> buffer(payload, payload + len);                                            \
        ptr->on_packet(header, buffer);                                                                 \
    }                                                                                                   \
    extern "C" void protocol_on_timer(::tcp_lab::sdk::Protocol* ptr, int timer_id)                      \
    {                                                                                                   \
        ptr->on_timer(timer_id);                                                                        \
    }

} // namespace tcp_lab::sdk

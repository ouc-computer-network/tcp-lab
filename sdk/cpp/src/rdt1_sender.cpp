#include "tcp_lab/sdk.hpp"

using namespace tcp_lab::sdk;

class Rdt1Sender final : public Protocol {
  public:
    void init() override {
        log("C++ RDT1 sender ready");
    }

    void on_app_data(const std::vector<uint8_t>& data) override {
        TcpHeader header{};
        std::vector<uint8_t> payload = data;
        log("RDT1 sender forwarding " + std::to_string(payload.size()) + " bytes");
        send_packet(header, payload);
    }
};

TCP_LAB_REGISTER_PROTOCOL(Rdt1Sender)

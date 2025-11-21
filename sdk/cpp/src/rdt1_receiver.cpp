#include "tcp_lab/sdk.hpp"

using namespace tcp_lab::sdk;

class Rdt1Receiver final : public Protocol {
  public:
    void init() override {
        log("C++ RDT1 receiver ready");
    }

    void on_packet(const TcpHeader&, const std::vector<uint8_t>& payload) override {
        log("RDT1 receiver delivering " + std::to_string(payload.size()) + " bytes");
        deliver_data(payload);
    }
};

TCP_LAB_REGISTER_PROTOCOL(Rdt1Receiver)

// Simple example C++ sender using the SDK.
// This file demonstrates how to implement a protocol in C++ and expose
// the standard entrypoints expected by the Rust loader.

#include <ouc/tcp/sdk/TransportProtocol.hpp>
#include <ouc/tcp/sdk/EntryPoints.hpp>

using namespace ouc::tcp::sdk;

class TestSender : public TransportProtocol {
public:
    void init(SystemContext& ctx) override {
        ctx.log("C++ TestSender init");
    }

    void onAppData(SystemContext& ctx, const std::vector<std::uint8_t>& data) override {
        TcpHeader h;
        h.seqNum = nextSeq_;
        Packet p{h, data};
        ctx.log("C++ TestSender sending packet");
        ctx.sendPacket(p);
        ++nextSeq_;
    }

    void onPacket(SystemContext& ctx, const Packet& packet) override {
        (void)ctx;
        (void)packet;
        // For a simple test sender we ignore incoming packets.
    }

    void onTimer(SystemContext& ctx, int timerId) override {
        (void)ctx;
        (void)timerId;
    }

private:
    std::uint32_t nextSeq_{0};
};

// Generate create_sender/destroy_sender and the sender_* C entrypoints
// that the Rust loader expects.
TCP_LAB_DEFINE_PROTOCOL_ENTRYPOINTS(TestSender);


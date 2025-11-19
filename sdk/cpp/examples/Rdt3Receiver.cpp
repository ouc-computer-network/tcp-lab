// C++ implementation of RDT 3.0 Receiver (Stop-and-Wait, lossy channel)
// Ported from sdk/java/src/com/ouc/tcp/impl/Rdt3Receiver.java

#include <ouc/tcp/sdk/TransportProtocol.hpp>
#include <ouc/tcp/sdk/EntryPoints.hpp>

using namespace ouc::tcp::sdk;

class Rdt3Receiver : public TransportProtocol {
public:
    void init(SystemContext& ctx) override {
        (void)ctx;
        expectedSeq_ = 0;
    }

    void onAppData(SystemContext& ctx, const std::vector<std::uint8_t>& data) override {
        (void)ctx;
        (void)data;
        // Receiver 不从应用层发送数据
    }

    void onPacket(SystemContext& ctx, const Packet& packet) override {
        const TcpHeader& h = packet.header;

        if (static_cast<int>(h.seqNum) == expectedSeq_) {
            ctx.log("RDT3Receiver: Received correct packet " + std::to_string(expectedSeq_));
            ctx.deliverData(packet.payload);

            // ACK 当前序号
            sendAck(ctx, expectedSeq_);

            expectedSeq_ = 1 - expectedSeq_;
        } else {
            ctx.log("RDT3Receiver: Duplicate/out-of-order packet " +
                    std::to_string(h.seqNum) + ", expected " + std::to_string(expectedSeq_));

            // 重发“上一个正确包”的 ACK：即 1 - expectedSeq_
            sendAck(ctx, 1 - expectedSeq_);
        }
    }

    void onTimer(SystemContext& ctx, int timerId) override {
        (void)ctx;
        (void)timerId;
        // Receiver 无定时器
    }

private:
    void sendAck(SystemContext& ctx, int ackNum) {
        TcpHeader ackH;
        ackH.ackNum = static_cast<std::uint32_t>(ackNum);
        ackH.flags = TcpHeader::Flags::ACK;
        Packet ackPkt{ackH, std::vector<std::uint8_t>{}};
        ctx.sendPacket(ackPkt);
    }

    int expectedSeq_{0};
};

// 为该 Receiver 生成标准 C 入口（create_sender / sender_*）
TCP_LAB_DEFINE_PROTOCOL_ENTRYPOINTS(Rdt3Receiver);



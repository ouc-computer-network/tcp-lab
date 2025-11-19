// C++ implementation of RDT 3.0 Sender (Stop-and-Wait with timer, lossy channel)
// Ported from sdk/java/src/com/ouc/tcp/impl/Rdt3Sender.java

#include <optional>

#include <ouc/tcp/sdk/TransportProtocol.hpp>
#include <ouc/tcp/sdk/EntryPoints.hpp>

using namespace ouc::tcp::sdk;

namespace {

static std::uint16_t rdt3_checksum(std::uint32_t seq, const std::vector<std::uint8_t>& data) {
    std::uint32_t sum = seq;
    for (auto b : data) {
        sum += static_cast<std::uint8_t>(b);
    }
    return static_cast<std::uint16_t>(sum & 0xFFFFu);
}

static bool rdt3_is_corrupted(const TcpHeader& h, const std::vector<std::uint8_t>& payload) {
    auto expected = rdt3_checksum(h.seqNum, payload);
    return expected != h.checksum;
}

} // namespace

class Rdt3Sender : public TransportProtocol {
public:
    void init(SystemContext& ctx) override {
        (void)ctx;
        nextSeq_ = 0;
        isWaiting_ = false;
        currentPacket_ = std::nullopt;
    }

    void onAppData(SystemContext& ctx, const std::vector<std::uint8_t>& data) override {
        // Stop-and-Wait: 如果还在等 ACK，则直接丢弃新的应用数据
        if (isWaiting_) {
            ctx.log("RDT3 Sender Busy: Dropping application data");
            return;
        }

        TcpHeader h;
        h.seqNum = static_cast<std::uint32_t>(nextSeq_);
        std::vector<std::uint8_t> payload = data;
        h.checksum = rdt3_checksum(h.seqNum, payload);

        currentPacket_ = Packet{h, payload};

        ctx.sendPacket(*currentPacket_);
        ctx.startTimer(3000, nextSeq_); // 3s timeout, ID = seq
        isWaiting_ = true;
    }

    void onPacket(SystemContext& ctx, const Packet& packet) override {
        if (!isWaiting_) return;

        const TcpHeader& h = packet.header;
        const std::vector<std::uint8_t>& payload = packet.payload;

        // RDT 3.0 (Lossy Channel)
        // 如果 ACK 损坏，或 ACK 号不是当前序号 => 立即重传并重启定时器
        bool corrupted = rdt3_is_corrupted(h, payload);

        if (corrupted || ((h.flags & TcpHeader::Flags::ACK) && h.ackNum != static_cast<std::uint32_t>(nextSeq_))) {
            ctx.log("Corrupted or Duplicate ACK. Retransmitting " + std::to_string(nextSeq_));
            ctx.cancelTimer(nextSeq_);
            if (currentPacket_) {
                ctx.sendPacket(*currentPacket_);
                ctx.startTimer(3000, nextSeq_);
            }
            return;
        }

        if ((h.flags & TcpHeader::Flags::ACK) && h.ackNum == static_cast<std::uint32_t>(nextSeq_)) {
            ctx.log("Received ACK " + std::to_string(nextSeq_));
            ctx.cancelTimer(nextSeq_);
            isWaiting_ = false;
            nextSeq_ = 1 - nextSeq_; // toggle 0/1
        }
    }

    void onTimer(SystemContext& ctx, int timerId) override {
        if (isWaiting_ && timerId == nextSeq_) {
            ctx.log("Timeout! Retransmitting seq " + std::to_string(nextSeq_));
            if (currentPacket_) {
                ctx.sendPacket(*currentPacket_);
                ctx.startTimer(3000, nextSeq_);
            }
        }
    }

private:
    int nextSeq_{0};
    bool isWaiting_{false};
    std::optional<Packet> currentPacket_{};
};

// 为该 Sender 生成标准 C 入口（create_sender / sender_*）
TCP_LAB_DEFINE_PROTOCOL_ENTRYPOINTS(Rdt3Sender);



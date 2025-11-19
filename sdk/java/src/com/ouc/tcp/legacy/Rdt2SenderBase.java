package com.ouc.tcp.legacy;

import com.ouc.tcp.sdk.*;

public abstract class Rdt2SenderBase extends TransportProtocol {
    private static final int TIMEOUT = 3000;
    protected int seq = 0;
    protected Packet current;
    protected boolean waiting = false;

    @Override
    public void onAppData(SystemContext ctx, byte[] data) {
        // 教材里的 Stop-and-Wait：发送端忙时，直接丢弃新到达的数据（或让上层重试）
        if (waiting) {
            ctx.log("RDT2 sender busy");
            return;
        }
        TcpHeader h = new TcpHeader();
        h.setSeqNum(seq);
        Rdt2Utils.attachChecksum(h, data);
        ctx.log("[Sender] Created packet with seq=" + seq + ", checksum=" + h.getChecksum() + ", data length=" + data.length);
        current = new Packet(h, data);
        send(ctx);
    }

    private void send(SystemContext ctx) {
        ctx.log("[Sender] Sending packet with seq=" + current.getHeader().getSeqNum());
        ctx.sendPacket(current);
        waiting = true;
    }

    @Override
    public void onPacket(SystemContext ctx, Packet packet) {
        TcpHeader h = packet.getHeader();
        // RDT 2.0/2.1: Check corruption on feedback
        // RDT 2.0 assumes feedback is error-free, but RDT 2.1 handles it.
        // We will use Rdt2Utils.isCorrupted even for ACK packets (checking header checksum)
        // Note: Our simulator corrupts the checksum field, so isCorrupted should detect it.
        boolean corrupted = Rdt2Utils.isCorrupted(h, packet.getPayload());
        
        ctx.log("[Sender] Rx: flags=" + h.getFlags() + ", ack=" + h.getAckNum() + ", corrupt=" + corrupted);

        if (corrupted) {
            ctx.log("[Sender] Corrupted feedback, retransmitting");
            send(ctx);
            return;
        }

        // Check for NAK (RST flag)
        if ((h.getFlags() & TcpHeader.Flags.RST) != 0) {
            ctx.log("[Sender] NAK received, retransmitting");
            send(ctx);
            return;
        }

        if (!h.isAck()) return;

        if (h.getAckNum() == seq) {
            ctx.log("[Sender] ACK " + seq + " received. Moving to next.");
            waiting = false;
            seq = 1 - seq;
        } else {
            ctx.log("[Sender] ACK " + h.getAckNum() + " received (expected " + seq + "). Retransmitting.");
            send(ctx);
        }
    }

    @Override
    public void onTimer(SystemContext ctx, int timerId) {
        // RDT 2.x doesn't use timer; ignore
    }

    protected boolean requiresNak() {
        return true;
    }
}

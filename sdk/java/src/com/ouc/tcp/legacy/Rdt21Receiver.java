package com.ouc.tcp.legacy;

import com.ouc.tcp.sdk.*;

public class Rdt21Receiver extends TransportProtocol {
    private int expectedSeq = 0;

    @Override
    public void onPacket(SystemContext ctx, Packet packet) {
        TcpHeader h = packet.getHeader();
        boolean corrupted = Rdt2Utils.isCorrupted(h, packet.getPayload());
        ctx.log("[Receiver] Rx Seq=" + h.getSeqNum() + ", Corrupt=" + corrupted + ", Expected=" + expectedSeq);

        if (corrupted) {
            // RDT 2.1: Corrupted packet -> Send NAK
            ctx.log("[Receiver] Corrupted. Sending NAK.");
            sendNak(ctx);
            return;
        }

        if (h.getSeqNum() != expectedSeq) {
            // RDT 2.1: Out of order (duplicate) -> Send ACK (re-ack)
            ctx.log("[Receiver] Duplicate Seq. Sending ACK " + h.getSeqNum());
            sendAck(ctx, (int) h.getSeqNum());
            return;
        }

        // Correct packet
        ctx.log("[Receiver] OK. Delivering.");
        ctx.deliverData(packet.getPayload());
        sendAck(ctx, expectedSeq);
        expectedSeq = 1 - expectedSeq;
    }

    private void sendNak(SystemContext ctx) {
        TcpHeader h = new TcpHeader();
        h.setFlags(TcpHeader.Flags.RST); // NAK
        Rdt2Utils.attachChecksum(h, new byte[0]);
        ctx.sendPacket(new Packet(h, new byte[0]));
    }

    private void sendAck(SystemContext ctx, int ackNum) {
        TcpHeader h = new TcpHeader();
        h.setAckNum(ackNum);
        h.setFlags(TcpHeader.Flags.ACK);
        Rdt2Utils.attachChecksum(h, new byte[0]);
        ctx.sendPacket(new Packet(h, new byte[0]));
    }

    @Override public void onAppData(SystemContext ctx, byte[] data) {}
    @Override public void onTimer(SystemContext ctx, int timerId) {}
    @Override public void init(SystemContext ctx) { expectedSeq = 0; }
}

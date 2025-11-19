package com.ouc.tcp.legacy;

import com.ouc.tcp.sdk.*;

public class Rdt22Receiver extends TransportProtocol {
    private int expectedSeq = 0;

    @Override
    public void onPacket(SystemContext ctx, Packet packet) {
        TcpHeader h = packet.getHeader();
        boolean corrupted = Rdt2Utils.isCorrupted(h, packet.getPayload());
        ctx.log("[Receiver] Rx Seq=" + h.getSeqNum() + ", Corrupt=" + corrupted + ", Expected=" + expectedSeq);

        if (corrupted || h.getSeqNum() != expectedSeq) {
            // RDT 2.2: Corrupted OR Wrong Seq -> Send ACK for the LAST correctly received packet (1 - expectedSeq)
            // Note: If we are waiting for 0, and get corrupt/1, we send ACK 1.
            int lastAck = 1 - expectedSeq;
            ctx.log("[Receiver] Corrupt/SeqErr. Sending ACK " + lastAck);
            sendAck(ctx, lastAck);
            return;
        }

        // Correct
        ctx.log("[Receiver] OK. Delivering.");
        ctx.deliverData(packet.getPayload());
        sendAck(ctx, expectedSeq);
        expectedSeq = 1 - expectedSeq;
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

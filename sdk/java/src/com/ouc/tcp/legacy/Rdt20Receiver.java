package com.ouc.tcp.legacy;

import com.ouc.tcp.sdk.*;

public class Rdt20Receiver extends TransportProtocol {
    private int expectedSeq = 0;
    private int lastAck = -1;

    @Override
    public void onPacket(SystemContext ctx, Packet packet) {
        TcpHeader h = packet.getHeader();
        // RDT 2.0 Receiver: 
        // 1. Check corruption.
        // 2. If corrupted -> Send NAK.
        // 3. If OK -> Deliver data, Send ACK.
        // Note: RDT 2.0 assumes no seq num check needed (implied strict order), 
        // but usually we verify it anyway or ignore it.
        // Given standard descriptions often omit seq num for 2.0, but 2.1 introduces it to handle ACK loss.
        // However, if we implement 2.0 strictly as "No ACK loss", then seq num is optional.
        // But our sender uses seq 0/1.
        // If we receive seq != expected, it means sender retransmitted? Or advanced?
        // In 2.0 channel (no ACK loss), sender only retransmits on NAK.
        // So if we receive duplicate, it must be because we sent NAK (due to corruption).
        
        boolean corrupted = Rdt2Utils.isCorrupted(h, packet.getPayload());
        ctx.log("[Receiver] Rx Seq=" + h.getSeqNum() + ", Corrupt=" + corrupted);
        
        if (corrupted) {
            ctx.log("[Receiver] Corrupted. Sending NAK.");
            sendNak(ctx);
            return;
        }

        // If we are strict 2.0, we might not check seq num, but let's handle duplicates gracefully if they happen.
        // Actually, if sender retransmits (e.g. due to our NAK), seq num will match expected.
        // If we receive unexpected seq, it's weird in 2.0 model (unless ACK lost, which isn't 2.0 model).
        // We will just process it.

        ctx.log("[Receiver] Data OK. Delivering.");
        ctx.deliverData(packet.getPayload());
        sendAck(ctx, (int) h.getSeqNum()); // ACK the received seq
        // expectedSeq = 1 - expectedSeq; // We don't really track this strictly in 2.0 if we just ACK what we see
    }

    private void sendAck(SystemContext ctx, int ackNum) {
        ctx.log("[Receiver] Sending ACK for seq=" + ackNum);
        TcpHeader h = new TcpHeader();
        h.setAckNum(ackNum);
        h.setFlags(TcpHeader.Flags.ACK);
        Rdt2Utils.attachChecksum(h, new byte[0]);
        ctx.sendPacket(new Packet(h, new byte[0]));
    }

    private void sendNak(SystemContext ctx) {
        ctx.log("[Receiver] Sending NAK");
        TcpHeader h = new TcpHeader();
        h.setAckNum(0); // Value doesn't matter much for NAK in 2.0
        h.setFlags(TcpHeader.Flags.RST); // using RST to represent NAK
        Rdt2Utils.attachChecksum(h, new byte[0]);
        ctx.sendPacket(new Packet(h, new byte[0]));
    }

    @Override public void onAppData(SystemContext ctx, byte[] data) {}
    @Override public void onTimer(SystemContext ctx, int timerId) {}
    @Override public void init(SystemContext ctx) { expectedSeq = 0; lastAck = -1; }
}

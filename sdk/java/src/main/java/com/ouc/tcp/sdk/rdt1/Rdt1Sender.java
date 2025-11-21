package com.ouc.tcp.sdk.rdt1;

import com.ouc.tcp.sdk.Packet;
import com.ouc.tcp.sdk.SystemContext;
import com.ouc.tcp.sdk.TcpHeader;
import com.ouc.tcp.sdk.TransportProtocol;

public final class Rdt1Sender implements TransportProtocol {
    @Override
    public void init(SystemContext ctx) {
        ctx.log("Java RDT1 sender ready");
    }

    @Override
    public void onPacket(SystemContext ctx, Packet packet) {
        ctx.log("RDT1 sender ignoring inbound packet seq=" + packet.getHeader().getSeqNum());
    }

    @Override
    public void onTimer(SystemContext ctx, int timerId) {
        ctx.log("RDT1 sender ignores timer " + timerId);
    }

    @Override
    public void onAppData(SystemContext ctx, byte[] data) {
        Packet packet = new Packet(new TcpHeader(), data);
        ctx.log("RDT1 sender sending " + data.length + " bytes");
        ctx.sendPacket(packet);
    }
}

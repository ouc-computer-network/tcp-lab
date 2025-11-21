package com.ouc.tcp.sdk.rdt1;

import com.ouc.tcp.sdk.Packet;
import com.ouc.tcp.sdk.SystemContext;
import com.ouc.tcp.sdk.TransportProtocol;

public final class Rdt1Receiver implements TransportProtocol {
    @Override
    public void init(SystemContext ctx) {
        ctx.log("Java RDT1 receiver ready");
    }

    @Override
    public void onPacket(SystemContext ctx, Packet packet) {
        ctx.log("RDT1 receiver delivering " + packet.getPayload().length + " bytes");
        ctx.deliverData(packet.getPayload());
    }

    @Override
    public void onTimer(SystemContext ctx, int timerId) {
        ctx.log("RDT1 receiver ignores timer " + timerId);
    }

    @Override
    public void onAppData(SystemContext ctx, byte[] data) {
        ctx.log("RDT1 receiver ignores outbound data of len " + data.length);
    }
}

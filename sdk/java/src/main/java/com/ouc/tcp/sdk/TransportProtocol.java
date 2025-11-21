package com.ouc.tcp.sdk;

public interface TransportProtocol {
    void init(SystemContext ctx);

    void onPacket(SystemContext ctx, Packet packet);

    void onTimer(SystemContext ctx, int timerId);

    void onAppData(SystemContext ctx, byte[] data);
}

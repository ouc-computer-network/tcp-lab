package com.ouc.tcp.sdk;

public final class Packet {
    private final TcpHeader header;
    private final byte[] payload;

    public Packet(TcpHeader header, byte[] payload) {
        this.header = header;
        this.payload = payload.clone();
    }

    public TcpHeader getHeader() {
        return header;
    }

    public byte[] getPayload() {
        return payload.clone();
    }
}

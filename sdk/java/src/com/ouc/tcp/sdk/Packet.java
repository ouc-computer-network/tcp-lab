package com.ouc.tcp.sdk;

import java.util.Arrays;

public class Packet {
    private TcpHeader header;
    private byte[] payload;

    public Packet(TcpHeader header, byte[] payload) {
        this.header = header;
        this.payload = payload;
    }

    public TcpHeader getHeader() {
        return header;
    }

    public byte[] getPayload() {
        return payload;
    }

    public int len() {
        return payload.length;
    }
    
    // Helper to create ACK
    public static Packet createAck(long seq, long ack, int window) {
        TcpHeader h = new TcpHeader();
        h.setSeqNum(seq);
        h.setAckNum(ack);
        h.setFlags(TcpHeader.Flags.ACK);
        h.setWindowSize(window);
        return new Packet(h, new byte[0]);
    }
}


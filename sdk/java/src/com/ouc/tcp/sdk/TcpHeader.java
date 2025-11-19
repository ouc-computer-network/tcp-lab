package com.ouc.tcp.sdk;

public class TcpHeader {
    public static class Flags {
        public static final byte FIN = 0x01;
        public static final byte SYN = 0x02;
        public static final byte RST = 0x04;
        public static final byte PSH = 0x08;
        public static final byte ACK = 0x10;
        public static final byte URG = 0x20;
    }

    private int srcPort;
    private int dstPort;
    private long seqNum; // Using long for unsigned 32-bit
    private long ackNum;
    private byte flags;
    private int windowSize;
    private int checksum;
    private int urgentPtr;

    public TcpHeader() {}

    public boolean isSyn() { return (flags & Flags.SYN) != 0; }
    public boolean isAck() { return (flags & Flags.ACK) != 0; }
    public boolean isFin() { return (flags & Flags.FIN) != 0; }

    // Getters and Setters
    public long getSeqNum() { return seqNum; }
    public void setSeqNum(long seqNum) { this.seqNum = seqNum; }

    public long getAckNum() { return ackNum; }
    public void setAckNum(long ackNum) { this.ackNum = ackNum; }

    public byte getFlags() { return flags; }
    public void setFlags(byte flags) { this.flags = flags; }

    public int getWindowSize() { return windowSize; }
    public void setWindowSize(int windowSize) { this.windowSize = windowSize; }
    
    public int getChecksum() { return checksum; }
    public void setChecksum(int checksum) { this.checksum = checksum; }

    public int getUrgentPtr() { return urgentPtr; }
    public void setUrgentPtr(int urgentPtr) { this.urgentPtr = urgentPtr; }
}


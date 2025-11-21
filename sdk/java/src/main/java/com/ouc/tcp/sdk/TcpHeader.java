package com.ouc.tcp.sdk;

public final class TcpHeader {
    private long seqNum;
    private long ackNum;
    private int flags;
    private int windowSize;
    private int checksum;

    public long getSeqNum() {
        return seqNum;
    }

    public void setSeqNum(long seqNum) {
        this.seqNum = seqNum;
    }

    public long getAckNum() {
        return ackNum;
    }

    public void setAckNum(long ackNum) {
        this.ackNum = ackNum;
    }

    public int getFlags() {
        return flags;
    }

    public void setFlags(int flags) {
        this.flags = flags;
    }

    public int getWindowSize() {
        return windowSize;
    }

    public void setWindowSize(int windowSize) {
        this.windowSize = windowSize;
    }

    public int getChecksum() {
        return checksum;
    }

    public void setChecksum(int checksum) {
        this.checksum = checksum;
    }
}

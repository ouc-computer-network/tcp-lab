package com.ouc.tcp.legacy;

import com.ouc.tcp.sdk.TcpHeader;

public final class Rdt2Utils {
    private Rdt2Utils() {}

    public static int checksum(long seq, byte[] data) {
        int sum = (int) seq;
        for (byte b : data) {
            sum += Byte.toUnsignedInt(b);
        }
        return sum & 0xFFFF;
    }

    public static boolean isCorrupted(TcpHeader header, byte[] payload) {
        int expected = checksum(header.getSeqNum(), payload);
        return expected != header.getChecksum();
    }

    public static void attachChecksum(TcpHeader header, byte[] payload) {
        header.setChecksum(checksum(header.getSeqNum(), payload));
    }
}

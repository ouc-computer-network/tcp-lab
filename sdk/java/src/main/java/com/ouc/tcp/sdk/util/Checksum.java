package com.ouc.tcp.sdk.util;

public final class Checksum {
    private Checksum() {}

    public static int internetChecksum(byte[] data) {
        int length = data.length;
        int i = 0;
        long sum = 0;

        while (length > 1) {
            int value = ((data[i] << 8) & 0xFF00) | (data[i + 1] & 0xFF);
            sum += value & 0xFFFF;
            if ((sum & 0xFFFF0000) != 0) {
                sum = (sum & 0xFFFF) + (sum >> 16);
            }
            i += 2;
            length -= 2;
        }

        if (length > 0) {
            sum += (data[i] << 8) & 0xFF00;
            if ((sum & 0xFFFF0000) != 0) {
                sum = (sum & 0xFFFF) + (sum >> 16);
            }
        }

        return (int) (~sum) & 0xFFFF;
    }
}

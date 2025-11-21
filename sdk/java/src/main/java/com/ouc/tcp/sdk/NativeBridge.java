package com.ouc.tcp.sdk;

final class NativeBridge {
    static {
        try {
            System.loadLibrary("tcp_lab_jni");
        } catch (UnsatisfiedLinkError e) {
            throw new RuntimeException("Failed to load tcp_lab_jni", e);
        }
    }

    private NativeBridge() {}

    static native void sendPacket(long seq, long ack, byte flags, int window, int checksum, int urgentPtr, byte[] payload);

    static native void startTimer(long delayMs, int timerId);

    static native void cancelTimer(int timerId);

    static native void deliverData(byte[] payload);

    static native void log(String message);

    static native long now();

    static native void recordMetric(String name, double value);
}

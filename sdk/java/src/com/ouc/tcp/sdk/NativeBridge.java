package com.ouc.tcp.sdk;

/**
 * JNI Bridge to the Rust Simulator.
 * This class is used internally and should not be used by students directly.
 */
public class NativeBridge {
    // Native registration is handled by the Rust host process using RegisterNatives.
    // No System.loadLibrary needed.

    // Native methods corresponding to SystemContext
    public static native void sendPacket(long seq, long ack, byte flags, int window, int checksum, int urgent, byte[] payload);
    public static native void startTimer(long delayMs, int timerId);
    public static native void cancelTimer(int timerId);
    public static native void deliverData(byte[] data);
    public static native void log(String message);
    public static native long now();
    public static native void recordMetric(String name, double value);
}


package com.ouc.tcp.sdk;

public interface SystemContext {
    void sendPacket(Packet packet);

    void startTimer(long delayMs, int timerId);

    void cancelTimer(int timerId);

    void deliverData(byte[] data);

    void log(String message);

    long now();

    void recordMetric(String name, double value);
}

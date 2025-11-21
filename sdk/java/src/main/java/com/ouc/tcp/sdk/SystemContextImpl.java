package com.ouc.tcp.sdk;

public final class SystemContextImpl implements SystemContext {
    @Override
    public void sendPacket(Packet packet) {
        var header = packet.getHeader();
        NativeBridge.sendPacket(
                header.getSeqNum(),
                header.getAckNum(),
                (byte) header.getFlags(),
                header.getWindowSize(),
                header.getChecksum(),
                header.getUrgentPointer(),
                packet.getPayload());
    }

    @Override
    public void startTimer(long delayMs, int timerId) {
        NativeBridge.startTimer(delayMs, timerId);
    }

    @Override
    public void cancelTimer(int timerId) {
        NativeBridge.cancelTimer(timerId);
    }

    @Override
    public void deliverData(byte[] data) {
        NativeBridge.deliverData(data);
    }

    @Override
    public void log(String message) {
        NativeBridge.log(message);
    }

    @Override
    public long now() {
        return NativeBridge.now();
    }

    @Override
    public void recordMetric(String name, double value) {
        NativeBridge.recordMetric(name, value);
    }
}

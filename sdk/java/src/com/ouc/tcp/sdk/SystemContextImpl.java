package com.ouc.tcp.sdk;

public class SystemContextImpl implements SystemContext {
    
    @Override
    public void sendPacket(Packet packet) {
        TcpHeader h = packet.getHeader();
        byte[] p = packet.getPayload();
        // Flatten the object structure for JNI simplicity
        NativeBridge.sendPacket(
            h.getSeqNum(),
            h.getAckNum(),
            h.getFlags(),
            h.getWindowSize(),
            h.getChecksum(),
            p
        );
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
}


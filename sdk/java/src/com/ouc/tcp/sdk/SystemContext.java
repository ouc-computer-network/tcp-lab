package com.ouc.tcp.sdk;

/**
 * The capability provided by the simulator to the student's protocol.
 * Students call these methods to interact with the network and application layer.
 */
public interface SystemContext {
    /**
     * Send a packet to the network (unreliable channel).
     */
    void sendPacket(Packet packet);

    /**
     * Start a timer.
     * @param delayMs Delay in milliseconds.
     * @param timerId A user-defined ID to identify this timer.
     */
    void startTimer(long delayMs, int timerId);

    /**
     * Cancel a running timer.
     */
    void cancelTimer(int timerId);

    /**
     * Deliver data to the Application Layer.
     * Call this when you have received valid, in-order data.
     */
    void deliverData(byte[] data);

    /**
     * Log a message to the simulator's debug output.
     */
    void log(String message);

    /**
     * Get current simulation time in ms.
     */
    long now();
}


package com.ouc.tcp.sdk;

/**
 * The interface that students must implement.
 * This replaces the old TCP_Sender_ADT / TCP_Receiver_ADT.
 */
public abstract class TransportProtocol {
    
    /**
     * Called when the simulation starts.
     */
    public void init(SystemContext ctx) {}

    /**
     * Called when a packet arrives from the network.
     */
    public abstract void onPacket(SystemContext ctx, Packet packet);

    /**
     * Called when a timer expires.
     */
    public abstract void onTimer(SystemContext ctx, int timerId);

    /**
     * Called when the Application Layer wants to send data reliably.
     * The protocol should encapsulate this data into packets and send them.
     * This corresponds to the old 'rdt_send'.
     */
    public abstract void onAppData(SystemContext ctx, byte[] data);
}


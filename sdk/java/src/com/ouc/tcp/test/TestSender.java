package com.ouc.tcp.test;

import com.ouc.tcp.sdk.*;
import java.util.Arrays;

public class TestSender extends TransportProtocol {

    @Override
    public void init(SystemContext ctx) {
        ctx.log("Java TestSender initialized!");
    }

    @Override
    public void onPacket(SystemContext ctx, Packet packet) {
        ctx.log("Java TestSender received ACK: " + packet.getHeader().getAckNum());
    }

    @Override
    public void onTimer(SystemContext ctx, int timerId) {
        ctx.log("Java TestSender timer expired: " + timerId);
    }

    @Override
    public void onAppData(SystemContext ctx, byte[] data) {
        ctx.log("Java TestSender sending data: " + Arrays.toString(data));
        
        TcpHeader h = new TcpHeader();
        h.setSeqNum(1);
        h.setAckNum(0);
        h.setFlags((byte)0);
        h.setWindowSize(100);
        
        Packet p = new Packet(h, data);
        ctx.sendPacket(p);
        
        // Start a timer just for fun
        ctx.startTimer(1000, 1);
    }
}


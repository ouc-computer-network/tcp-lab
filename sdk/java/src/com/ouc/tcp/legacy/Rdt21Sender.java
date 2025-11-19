package com.ouc.tcp.legacy;

public class Rdt21Sender extends Rdt2SenderBase {
    @Override
    protected boolean requiresNak() {
        return false;
    }
}

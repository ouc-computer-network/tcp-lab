package com.ouc.tcp.legacy;

public class Rdt22Sender extends Rdt2SenderBase {
    @Override
    protected boolean requiresNak() {
        return false;
    }
}

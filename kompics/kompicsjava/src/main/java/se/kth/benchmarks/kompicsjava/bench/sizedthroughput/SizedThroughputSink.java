
package se.kth.benchmarks.kompicsjava.bench.sizedthroughput;

import se.kth.benchmarks.kompicsjava.bench.netthroughputpingpong.Ping;
import se.kth.benchmarks.kompicsjava.bench.netthroughputpingpong.Pong;
import se.kth.benchmarks.kompicsjava.bench.sizedthroughput.SizedThroughputMessage;
import se.kth.benchmarks.kompicsjava.bench.streamingwindows.StreamSink;
import se.sics.kompics.ComponentDefinition;
import se.sics.kompics.Positive;
import se.sics.kompics.network.Network;
import se.sics.kompics.ClassMatchedHandler;
import se.sics.kompics.Handler;
import se.sics.kompics.KompicsEvent;
import se.sics.kompics.Start;
import se.kth.benchmarks.kompics.ConfigKeys;
import java.util.concurrent.CountDownLatch;

import se.kth.benchmarks.kompicsjava.net.NetAddress;
import se.kth.benchmarks.kompicsjava.net.NetMessage;

public class SizedThroughputSink extends ComponentDefinition {
    private final Positive<Network> net = requires(Network.class);

    final int id;
    private final NetAddress selfAddr;
    NetAddress upstream;
    int batchSize;
    int received = 0;

    public SizedThroughputSink(Init init) {

        this.selfAddr = config().getValue(ConfigKeys.SELF_ADDR_KEY, NetAddress.class);
        this.id = init.id;
        this.batchSize = init.batchSize;
        // subscriptions
        subscribe(messageHandler, net);
    }

    static class Ack implements KompicsEvent {
        int id;
        Ack(int id) {
            this.id = id;
        }
    };

    /*
    private Handler<SizedThroughputMessage> messageHandler = new Handler<SizedThroughputMessage>() {
        @Override
        public void handle(SizedThroughputMessage event) {
            if (event.id == id) {
                received += event.aux; // ensures deserialization isn't omitted by optimization
                if (received >= batchSize) {
                    received = 0;
                    trigger(NetMessage.viaTCP(selfAddr, new Ack(id)), net);
                }
            }
        }
    }; */

    private ClassMatchedHandler<SizedThroughputMessage, NetMessage> messageHandler
            = new ClassMatchedHandler<SizedThroughputMessage, NetMessage>() {
        @Override
        public void handle(SizedThroughputMessage content, NetMessage context) {
            if (content.id == id) {
                received += content.aux; // ensures deserialization isn't omitted by optimization
                if (received >= batchSize) {
                    received = 0;
                    trigger(context.reply(selfAddr, new Ack(id)), net);
                }
            }
        }
    };

    public static class Init extends se.sics.kompics.Init<StreamSink> {
        final int id;
        final int batchSize;

        public Init(int id, int batchSize) {
            this.id = id;
            this.batchSize = batchSize;
        }
    }
}

package se.kth.benchmarks.kompicsjava.bench.fibonacci;

import se.kth.benchmarks.kompicsjava.bench.fibonacci.FibonacciPort;
import se.sics.kompics.ComponentDefinition;
import se.sics.kompics.Handler;
import se.sics.kompics.Fault;
import se.sics.kompics.Positive;
import se.sics.kompics.Negative;
import se.sics.kompics.Kill;
import se.sics.kompics.Killed;
import se.sics.kompics.Start;
import se.sics.kompics.Component;
import se.sics.kompics.Channel;
import java.util.Optional;
import java.util.concurrent.CountDownLatch;

import java.util.UUID;
import se.kth.benchmarks.kompicsjava.bench.fibonacci.Messages.*;

public class FibonacciComponent extends ComponentDefinition {

    final Negative<FibonacciPort> parentPort = provides(FibonacciPort.class);
    final Positive<FibonacciPort> childPort = requires(FibonacciPort.class);

    final Optional<CountDownLatch> latchOpt;

    final UUID myId = this.id();

    public FibonacciComponent(Init init) {
        this.latchOpt = init.latchOpt;
        // subscriptions
        subscribe(killedHandler, control);
        subscribe(requestHandler, parentPort);
        subscribe(responseHandler, childPort);
    }

    private long result = 0l;
    private int numResponses = 0;

    private Component f1 = null;
    private Component f2 = null;

    final Handler<Killed> killedHandler = new Handler<Killed>() {
        @Override
        public void handle(Killed event) {
            if ((f1 != null) && (event.component.id() == f1.id())) {
                f1 = null;
            } else if ((f2 != null) && (event.component.id() == f2.id())) {
                f2 = null;
            }
            maybeSuicide();
        }
    };

    final Handler<FibRequest> requestHandler = new Handler<FibRequest>() {
        @Override
        public void handle(FibRequest event) {
            if (event.id == myId) {
                int n = event.n;
                logger.debug("Got FibRequest with n={}", n);
                if (n <= 2) {
                    sendResult(1L);
                    suicide(); // shut down immediately
                } else {
                    f1 = create(FibonacciComponent.class, new Init(Optional.empty()));
                    f2 = create(FibonacciComponent.class, new Init(Optional.empty()));
                    Negative<FibonacciPort> childPortOuter = childPort.getPair();
                    connect(childPortOuter, f1.getPositive(FibonacciPort.class), Channel.TWO_WAY);
                    connect(childPortOuter, f2.getPositive(FibonacciPort.class), Channel.TWO_WAY);
                    trigger(Start.event, f1.getControl());
                    trigger(Start.event, f2.getControl());
                    trigger(new FibRequest(n - 1, f1.id()), childPort);
                    trigger(new FibRequest(n - 2, f2.id()), childPort);
                }
            }
        }
    };

    final Handler<FibResponse> responseHandler = new Handler<FibResponse>() {
        @Override
        public void handle(FibResponse event) {
            logger.debug("Got FibResponse with value={}", event.value);
            numResponses++;
            result += event.value;

            if (numResponses == 2) {
                sendResult(result);
                maybeSuicide();
            }
        }
    };

    private void sendResult(long value) {
        if (this.latchOpt.isPresent()) {
            CountDownLatch latch = latchOpt.get();
            latch.countDown();
            logger.info("Final value was {}", value);
        } else {
            trigger(new FibResponse(value), parentPort);
        }
    }

    private void maybeSuicide() {
        if ((numResponses == 2) && (f1 == null) && (f2 == null)) {
            logger.debug("Both children died, shutting down");
            suicide();
        }
    }

    public static class Init extends se.sics.kompics.Init<FibonacciComponent> {
        public final Optional<CountDownLatch> latchOpt;

        public Init(Optional<CountDownLatch> latchOpt) {
            this.latchOpt = latchOpt;
        }
    }
}

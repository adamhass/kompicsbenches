package se.kth.benchmarks.kompicsjava.bench.atomicregister.events;

import se.sics.kompics.KompicsEvent;

public class Ack implements KompicsEvent{
    public long key;
    public int rid;
    public int run_id;
    public Ack(int run_id, long key, int rid){ this.key = key; this.rid = rid; this.run_id = run_id; }
}

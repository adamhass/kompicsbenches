use super::*;

use crate::bench::messages::SizedThroughputMessage;
use kompact::prelude::*;
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use synchronoise::CountdownEvent;

#[derive(Clone, Copy)]
pub struct SizedThroughputParameters {
    message_size: u64,
    batch_size: u64,
    number_of_batches: u64,
    number_of_pairs: u64,
}

impl SizedThroughputParameters {
    fn new(
        message_size: u64,
        batch_size: u64,
        number_of_batches: u64,
        number_of_pairs: u64,
    ) -> SizedThroughputParameters {
        SizedThroughputParameters {
            message_size,
            batch_size,
            number_of_batches,
            number_of_pairs,
        }
    }
}

pub struct SizedRefs(Vec<ActorPath>);

#[derive(Default)]
pub struct SizedThroughputBenchmark;

impl DistributedBenchmark for SizedThroughputBenchmark {
    type MasterConf = SizedThroughputParameters;
    type ClientConf = SizedThroughputParameters;
    type ClientData = SizedRefs;

    type Master = SizedThroughputMaster;
    type Client = SizedThroughputClient;

    const LABEL: &'static str = "SizedThroughput";

    fn new_master() -> Self::Master {
        SizedThroughputMaster::new()
    }

    fn msg_to_master_conf(
        msg: Box<dyn (::protobuf::Message)>,
    ) -> Result<Self::MasterConf, BenchmarkError> {
        downcast_msg!(msg; SizedThroughputParameters)
    }

    fn new_client() -> Self::Client {
        SizedThroughputClient::new()
    }
    fn str_to_client_conf(str: String) -> Result<Self::ClientConf, BenchmarkError> {
        let split: Vec<_> = str.split(',').collect();
        if split.len() != 4 {
            Err(BenchmarkError::InvalidMessage(format!(
                "String '{}' does not represent a client conf!",
                str
            )))
        } else {
            let message_size = split[0];
            let message_size = message_size.parse::<u64>().map_err(|e| {
                BenchmarkError::InvalidMessage(format!(
                    "String '{}' does not represent a client conf: {:?}",
                    str, e
                ))
            })?;
            let batch_size_str = split[1];
            let batch_size = batch_size_str.parse::<u64>().map_err(|e| {
                BenchmarkError::InvalidMessage(format!(
                    "String '{}' does not represent a client conf: {:?}",
                    str, e
                ))
            })?;
            let number_of_batches_str = split[2];
            let number_of_batches = number_of_batches_str.parse::<u64>().map_err(|e| {
                BenchmarkError::InvalidMessage(format!(
                    "String '{}' does not represent a client conf: {:?}",
                    str, e
                ))
            })?;
            let number_of_pairs = split[3];
            let number_of_pairs = number_of_pairs.parse::<u64>().map_err(|e| {
                BenchmarkError::InvalidMessage(format!(
                    "String '{}' does not represent a client conf: {:?}",
                    str, e
                ))
            })?;
            Ok(SizedThroughputParameters::new(
                message_size,
                batch_size,
                number_of_batches,
                number_of_pairs,
            ))
        }
    }

    fn str_to_client_data(str: String) -> Result<Self::ClientData, BenchmarkError> {
        let res: Result<Vec<_>, _> = str.split(',').map(|s| ActorPath::from_str(s)).collect();
        res.map(|paths| SizedRefs(paths)).map_err(|e| {
            BenchmarkError::InvalidMessage(format!("Could not read client data: {}", e))
        })
    }

    fn client_conf_to_str(c: Self::ClientConf) -> String {
        format!(
            "{},{},{},{}",
            c.message_size, c.batch_size, c.number_of_batches, c.number_of_pairs,
        )
    }
    fn client_data_to_str(d: Self::ClientData) -> String {
        d.0.into_iter()
            .map(|path| path.to_string())
            .collect::<Vec<String>>()
            .join(",")
    }
}

const REG_TIMEOUT: Duration = Duration::from_secs(6);
const FLUSH_TIMEOUT: Duration = Duration::from_secs(60);

pub struct SizedThroughputMaster {
    params: Option<SizedThroughputParameters>,
    system: Option<KompactSystem>,
    latch: Option<Arc<CountdownEvent>>,
    sources: Vec<(u64, Arc<Component<SizedThroughputSource>>)>,
    //sinks: Vec<Arc<Component<SizedThroughputSink>>>,
    source_refs: Vec<ActorRef<SourceMsg>>,
}

impl SizedThroughputMaster {
    fn new() -> SizedThroughputMaster {
        SizedThroughputMaster {
            params: None,
            system: None,
            latch: None,
            sources: Vec::new(),
            // sinks: Vec::new(),
            source_refs: Vec::new(),
        }
    }
}

impl DistributedBenchmarkMaster for SizedThroughputMaster {
    type MasterConf = SizedThroughputParameters;
    type ClientData = SizedRefs;
    type ClientConf = SizedThroughputParameters;

    fn setup(
        &mut self,
        c: Self::MasterConf,
        _m: &DeploymentMetaData,
    ) -> Result<Self::ClientConf, BenchmarkError> {
        let system = crate::kompact_system_provider::global().new_remote_system("SizedThroughput");

        let latch = Arc::new(CountdownEvent::new(c.number_of_pairs as usize));

        for pid in 0..c.number_of_pairs {
            let (source, req_f) =
                system.create_and_register(|| SizedThroughputSource::with(c, latch.clone()));
            // let source_path = req_f.wait_expect(REG_TIMEOUT, "Source failed to register!");
            system
                .start_notify(&source)
                .wait_timeout(REG_TIMEOUT)
                .expect("Source failed to start!");
            self.source_refs.push(source.actor_ref().clone());
            self.sources.push((pid, source));
        }
        self.latch = Some(latch);
        self.system = Some(system);
        self.params = Some(c);
        Ok(c)
    }

    fn prepare_iteration(&mut self, mut d: Vec<Self::ClientData>) -> () {
        let sinks = &mut d[0].0;
        assert_eq!(
            sinks.len(),
            self.source_refs.len(),
            "Same amount of sinks as sources"
        );
        for source in &self.source_refs {
            // Tell all the sources who their target is
            source.tell(SourceMsg::Target(sinks.pop().unwrap()));
        }
        // The sources will send a prepare message to the sink and then decrement the latch
        // When they receive the Ack
        // This way all the networking and buffers are allocated and ready for usage when we
        // run the iteration.
        if let Some(latch) = &mut self.latch {
            latch.wait_timeout(FLUSH_TIMEOUT);
            if let Some(l) = Arc::get_mut(latch) {
                l.reset();
            }
        }
    }

    fn run_iteration(&mut self) -> () {
        if let Some(ref _system) = self.system {
            let latch = self.latch.take().unwrap();
            self.source_refs.iter().for_each(|source_ref| {
                source_ref.tell(SourceMsg::Run);
            });
            latch.wait();
        } else {
            unimplemented!()
        }
    }
    fn cleanup_iteration(&mut self, last_iteration: bool, _exec_time_millis: f64) -> () {
        if last_iteration {
            println!("Cleaning up sinks for SizedThroughput, last iteration");
            if let Some(system) = self.system.take() {
                let mut kill_futures = Vec::new();
                for (_, source) in self.sources.drain(..) {
                    let kf = system.kill_notify(source);
                    kill_futures.push(kf);
                }
                for kf in kill_futures {
                    kf.wait_timeout(Duration::from_millis(1000))
                        .expect("Source Actor never died!");
                }
                system
                    .shutdown()
                    .expect("Kompact didn't shut down properly");
            }
        } else {
            println!("Cleaning up sinks for SizedThroughput iteration, doing nothing");
        }
    }
}

pub struct SizedThroughputClient {
    system: Option<KompactSystem>,
    sinks: Vec<Arc<Component<SizedThroughputSink>>>,
}
impl SizedThroughputClient {
    fn new() -> SizedThroughputClient {
        SizedThroughputClient {
            system: None,
            sinks: Vec::new(),
        }
    }
}

impl DistributedBenchmarkClient for SizedThroughputClient {
    type ClientConf = SizedThroughputParameters;
    type ClientData = SizedRefs;

    fn setup(&mut self, c: Self::ClientConf) -> Self::ClientData {
        println!("Setting up Sinks.");
        let system = crate::kompact_system_provider::global().new_remote_system("SizedThroughput");

        let mut sinks: Vec<ActorPath> = Vec::new();
        for _ in 0..c.number_of_pairs {
            let (sink, reg_f) = system.create_and_register(|| SizedThroughputSink::new());
            let sink_path = reg_f.wait_expect(REG_TIMEOUT, "Sink failed to register!");

            system
                .start_notify(&sink)
                .wait_timeout(REG_TIMEOUT)
                .expect("Sink failed to start!");

            self.sinks.push(sink);
            sinks.push(sink_path);
        }
        self.system = Some(system);
        SizedRefs(sinks)
    }

    fn prepare_iteration(&mut self) -> () {
        // nothing to do
        println!("Preparing sinks for SizedThroughput iteration");
    }

    fn cleanup_iteration(&mut self, last_iteration: bool) -> () {
        if last_iteration {
            println!("Cleaning up sinks for SizedThroughput, last iteration");
            if let Some(system) = self.system.take() {
                let mut kill_futures = Vec::new();
                for sink in self.sinks.drain(..) {
                    let kf = system.kill_notify(sink);
                    kill_futures.push(kf);
                }
                for kf in kill_futures {
                    kf.wait_timeout(Duration::from_millis(1000))
                        .expect("Sink Actor never died!");
                }
                system
                    .shutdown()
                    .expect("Kompact didn't shut down properly");
            }
        } else {
            println!("Cleaning up sinks for SizedThroughput iteration, doing nothing");
        }
    }
}

#[derive(ComponentDefinition)]
pub struct SizedThroughputSource {
    ctx: ComponentContext<Self>,
    latch: Arc<CountdownEvent>,
    downstream: Option<ActorPath>,
    message_size: u64,
    batch_size: u64,
    number_of_batches: u64,
    current_batch: u64,
}

impl SizedThroughputSource {
    pub fn with(
        params: SizedThroughputParameters,
        latch: Arc<CountdownEvent>,
    ) -> SizedThroughputSource {
        SizedThroughputSource {
            ctx: ComponentContext::uninitialised(),
            latch,
            downstream: None,
            message_size: params.message_size,
            batch_size: params.batch_size,
            number_of_batches: params.number_of_batches,
            current_batch: 0,
        }
    }

    fn send(&mut self) {
        for _ in 0..self.batch_size {
            if let Some(sink) = &self.downstream {
                let message = SizedThroughputMessage::new(self.message_size as usize);
                sink.tell_serialised(SinkMsg::Message(message), self)
                    .expect("serialise");
            }
        }
    }
}

ignore_lifecycle!(SizedThroughputSource);

impl NetworkActor for SizedThroughputSource {
    type Message = SourceMsg;
    type Deserialiser = SourceMsg;

    fn receive(&mut self, _: Option<ActorPath>, msg: Self::Message) -> Handled {
        match msg {
            SourceMsg::Ack => {
                // The sink has received the batch and is ready for the next one
                self.current_batch += 1;
                if self.current_batch >= self.number_of_batches {
                    // Finished
                    self.latch.decrement().expect("Decrement Latch");
                } else {
                    // Send the next batch
                    self.send();
                }
            }
            SourceMsg::Run => {
                // We start the experiment, set current_batch to 0
                self.current_batch = 0;
                self.send();
            }
            SourceMsg::Target(path) => {
                // Make target prepare for receiving the batch_size
                path.tell_serialised(SinkMsg::Prepare(self.batch_size), self)
                    .expect("serialise");
                self.downstream = Some(path);
            }
            SourceMsg::Ready => {
                // Ready received from the Sink
                // Decrement the latch and wait for Run message before we start.
                let _ = self.latch.decrement();
            }
        }
        Handled::Ok
    }
}

#[derive(ComponentDefinition)]
pub struct SizedThroughputSink {
    ctx: ComponentContext<Self>,
    batch_size: u64,
    received: u64,
}
impl SizedThroughputSink {
    pub fn new() -> SizedThroughputSink {
        SizedThroughputSink {
            ctx: ComponentContext::uninitialised(),
            batch_size: 0,
            received: 0,
        }
    }
}

ignore_lifecycle!(SizedThroughputSink);

impl NetworkActor for SizedThroughputSink {
    type Message = SinkMsg;
    type Deserialiser = SinkMsg;

    fn receive(&mut self, sender: Option<ActorPath>, msg: Self::Message) -> Handled {
        match msg {
            SinkMsg::Message(_) => {
                self.received += 1;
                if self.received == self.batch_size {
                    if let Some(source) = sender {
                        // Ack the batch and reset the received counter
                        source.tell(SourceMsg::Ack, self);
                        self.received = 0;
                    } else {
                        panic!("No source for the Ack message!")
                    }
                }
            }
            SinkMsg::Prepare(batch_size) => {
                self.batch_size = batch_size;
                self.received = 0;
                if let Some(source) = sender {
                    source.tell(SourceMsg::Ready, self);
                } else {
                    panic!("No source for the Ready message!")
                }
            }
        }
        Handled::Ok
    }
}

#[derive(Clone, Debug)]
pub enum SourceMsg {
    Run,
    Target(ActorPath),
    Ack,
    Ready,
}

impl SourceMsg {
    const SERID: SerId = serialiser_ids::STP_SOURCE_ID;
    const RUN_FLAG: u8 = 1u8;
    const ACK_FLAG: u8 = 2u8;
    const TARGET_FLAG: u8 = 3u8;
    const READY_FLAG: u8 = 4u8;
}

impl Serialisable for SourceMsg {
    fn ser_id(&self) -> SerId {
        Self::SERID
    }
    fn size_hint(&self) -> Option<usize> {
        match self {
            SourceMsg::Run => None, // don't serialise
            SourceMsg::Ack => None,
            SourceMsg::Ready => None,
            SourceMsg::Target(path) => path.size_hint(),
        }
    }
    fn serialise(&self, buf: &mut dyn BufMut) -> Result<(), SerError> {
        match self {
            SourceMsg::Run => {
                buf.put_u8(Self::RUN_FLAG);
            }
            SourceMsg::Ack => {
                buf.put_u8(Self::ACK_FLAG);
            }
            SourceMsg::Ready => {
                buf.put_u8(Self::READY_FLAG);
            }
            SourceMsg::Target(path) => {
                buf.put_u8(Self::TARGET_FLAG);
                path.serialise(buf)?;
            }
        }
        Ok(())
    }

    fn local(self: Box<Self>) -> Result<Box<dyn Any + Send>, Box<dyn Serialisable>> {
        Ok(self)
    }
}
impl Deserialiser<SourceMsg> for SourceMsg {
    const SER_ID: SerId = Self::SERID;
    fn deserialise(buf: &mut dyn Buf) -> Result<SourceMsg, SerError> {
        match buf.get_u8() {
            Self::ACK_FLAG => Ok(Self::Ack),
            Self::RUN_FLAG => Ok(Self::Run),
            Self::READY_FLAG => Ok(Self::Ready),
            Self::TARGET_FLAG => Ok(Self::Target(ActorPath::deserialise(buf)?)),
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SinkMsg {
    Prepare(u64),
    Message(SizedThroughputMessage),
}

impl SinkMsg {
    const SERID: SerId = serialiser_ids::STP_SINK_ID;
    const PREPARE_FLAG: u8 = 1u8;
    const MESSAGE_FLAG: u8 = 2u8;
}

impl Serialisable for SinkMsg {
    fn ser_id(&self) -> SerId {
        Self::SERID
    }
    fn size_hint(&self) -> Option<usize> {
        match self {
            SinkMsg::Prepare(_) => Some(8),
            SinkMsg::Message(msg) => msg.size_hint(),
        }
    }
    fn serialise(&self, buf: &mut dyn BufMut) -> Result<(), SerError> {
        match self {
            SinkMsg::Prepare(size) => {
                buf.put_u8(Self::PREPARE_FLAG);
                buf.put_u64(*size);
            }
            SinkMsg::Message(msg) => {
                buf.put_u8(Self::MESSAGE_FLAG);
                msg.serialise(buf)?;
            }
        }
        Ok(())
    }

    fn local(self: Box<Self>) -> Result<Box<dyn Any + Send>, Box<dyn Serialisable>> {
        Ok(self)
    }
}
impl Deserialiser<SinkMsg> for SinkMsg {
    const SER_ID: SerId = Self::SERID;
    fn deserialise(buf: &mut dyn Buf) -> Result<SinkMsg, SerError> {
        match buf.get_u8() {
            Self::PREPARE_FLAG => Ok(Self::Prepare(buf.get_u64())),
            Self::MESSAGE_FLAG => Ok(Self::Message(SizedThroughputMessage::deserialise(buf)?)),
            _ => unimplemented!(),
        }
    }
}

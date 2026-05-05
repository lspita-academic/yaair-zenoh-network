use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use thiserror::Error;
use yaair::yaair::{
    messages::{inbound::InboundMessage, serializer::Serializer},
    network::Network,
};
use zenoh_pico::{
    keyexpr::KeyExpr,
    sample::Sample,
    session::{
        Session,
        pubsub::{Publisher, Subscriber},
    },
    timestamp::Timestamp,
    zbytes::IntoZBytes,
    zid::ZId,
    zvalue::ZValue,
};

struct Message {
    payload: Vec<u8>,
    timestamp: SystemTime,
}

impl Message {
    pub fn new(payload: Vec<u8>) -> Self {
        let timestamp = SystemTime::now();
        Self { payload, timestamp }
    }
}

impl From<&Sample> for Message {
    fn from(value: &Sample) -> Self {
        Self::new(value.payload().owned_bytes())
    }
}

struct AtomicMessagesStore(Mutex<HashMap<ZId, Message>>);

impl AtomicMessagesStore {
    pub fn store(&mut self, zid: ZId, message: Message) {
        self.0.lock().unwrap().insert(zid, message);
    }
}

pub struct ZenohPicoNetwork<S> {
    serializer: S,
    publisher: Publisher,
    subscriber: Subscriber,
    messages: Arc<AtomicMessagesStore>,
}

/// NOTE: sample source info api is marked as unstable, so instead we send the
/// session zid in the payload to recognize peers.
impl<S: Serializer> ZenohPicoNetwork<S> {
    unsafe extern "C" fn on_message(
        sample: *const <Sample as ZValue>::Value,
        context: *mut AtomicMessagesStore,
    ) {
        let sample = &unsafe { *sample };
    }

    pub fn new(session: &Session, base_keyexpr: &KeyExpr) {}
}

impl<S: Serializer> Network<ZId, S> for ZenohPicoNetwork<S> {
    fn prepare_outbound(&mut self, outbound_message: Vec<u8>) {
        todo!()
    }

    fn prepare_inbound(&mut self) -> InboundMessage<ZId> {
        todo!()
    }
}

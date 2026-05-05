pub(self) mod atomic;
pub(self) mod message;

use std::sync::Arc;

use yaair::yaair::{
    messages::{inbound::InboundMessage, serializer::Serializer},
    network::Network,
};
use zenoh_pico::{
    keyexpr::KeyExpr,
    result::ZenohResult,
    sample::{Sample, SampleClosure},
    session::{
        Session,
        pubsub::{Publisher, Subscriber},
    },
    zid::ZId,
    zvalue::{ZClone, ZClosure, ZValue},
};

use crate::message::{AtomicMessageSerializer, AtomicMessagesStore, Message};

struct NetworkContext<S> {
    messages: AtomicMessagesStore,
    serializer: AtomicMessageSerializer<S>,
}

pub struct ZenohPicoNetwork<'a, S> {
    session: &'a Session,
    messages_publisher: Publisher,
    messages_subscriber: Subscriber,
    context: Arc<NetworkContext<S>>,
}

/// NOTE: sample source info api is marked as unstable, so instead we send the
/// session zid in the payload to recognize peers.
impl<'a, S: Serializer> ZenohPicoNetwork<'a, S> {
    pub fn new(session: &'a Session, base_keyexpr: &KeyExpr, serializer: S) -> ZenohResult<Self> {
        let context = Arc::new(NetworkContext {
            messages: AtomicMessagesStore::new(),
            serializer: AtomicMessageSerializer::new(serializer),
        });

        let zid = session.zid();
        let messages_publisher = session.declare_publisher(
            &base_keyexpr.join_autocanonize(&KeyExpr::new(&zid.to_string())?)?,
            None,
        )?;
        let messages_subscriber = session.declare_subscriber(
            &base_keyexpr.join_autocanonize(&KeyExpr::new("*")?)?,
            SampleClosure::from_callback(Self::on_message, Some(context.clone()))?,
            None,
        )?;

        Ok(Self {
            session,
            messages_publisher,
            messages_subscriber,
            context,
        })
    }

    unsafe extern "C" fn on_message(
        sample: *const <Sample as ZValue>::Value,
        context: *const NetworkContext<S>,
    ) {
        let sample = Sample::zclone(sample);
        let context = unsafe { &*context };

        let payload_bytes = sample.payload().owned_bytes();
        let packet = match context.serializer.deserialize_packet(payload_bytes) {
            Ok(p) => p,
            Err(e) => {
                log::warn!("Failed to deserialize message packet: {e}");
                return;
            }
        };

        let zid = packet.sender();
        let message = Message::from(packet);
        if let Err(e) = context.messages.store(zid, message) {
            log::warn!("Failed to store message: {e}");
            return;
        }
    }
}

impl<S: Serializer> Network<ZId, S> for ZenohPicoNetwork<'_, S> {
    fn prepare_outbound(&mut self, outbound_message: Vec<u8>) {
        todo!()
    }

    fn prepare_inbound(&mut self) -> InboundMessage<ZId> {
        todo!()
    }
}

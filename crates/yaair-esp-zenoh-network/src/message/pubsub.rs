use std::sync::Arc;

use yaair::yaair::messages::{outbound::OutboundMessage, serializer::Serializer};
use zenoh_pico::{
    keyexpr::KeyExpr,
    result::{ZenohError, ZenohResult},
    sample::{Sample, SampleClosure},
    session::{
        Session,
        pubsub::{Publisher, Subscriber},
    },
    zbytes::TryIntoZBytes,
    zid::ZId,
    zvalue::{ZClone, ZClosure, ZValue},
};

use crate::NetworkContext;

pub struct MessageSubscriber {
    _subscriber: Subscriber, // keep alive
}

impl MessageSubscriber {
    pub fn new<S: Serializer>(
        session: &Session,
        base_keyexpr: &KeyExpr,
        context: Arc<NetworkContext<S>>,
    ) -> ZenohResult<Self> {
        let subscriber = session.declare_subscriber(
            &base_keyexpr.join_autocanonize(&KeyExpr::new("*")?)?,
            SampleClosure::from_callback(Self::on_message::<S>, Some(context.clone()))?,
            None,
        )?;
        Ok(Self {
            _subscriber: subscriber,
        })
    }

    unsafe extern "C" fn on_message<S: Serializer>(
        sample: *const <Sample as ZValue>::Value,
        context: *const NetworkContext<S>,
    ) {
        log::info!("Received message");
        let sample = Sample::zclone(sample);
        let context = unsafe { &*context };

        let payload_bytes = sample.payload().owned_bytes();
        log::debug!("Payload size: {}", payload_bytes.len());
        let outbound_message: OutboundMessage<ZId> =
            match context.serializer.deserialize(&payload_bytes) {
                Ok(p) => p,
                Err(e) => {
                    log::warn!("Failed to deserialize message packet: {e}");
                    return;
                }
            };

        log::info!("Sender: {}", outbound_message.sender);
        match context
            .messages
            .store(outbound_message.sender, outbound_message.into())
        {
            Ok(_) => log::info!("Message stored successfully"),
            Err(e) => log::warn!("Failed to store message: {e}"),
        }
    }
}

pub struct MessagePublisher {
    publisher: Publisher,
}

impl MessagePublisher {
    pub fn new(session: &Session, base_keyexpr: &KeyExpr) -> ZenohResult<Self> {
        let zid = session.zid();
        let publisher = session.declare_publisher(
            &base_keyexpr.join_autocanonize(&KeyExpr::new(&zid.to_string())?)?,
            None,
        )?;
        Ok(Self { publisher })
    }

    pub fn publisher(&self) -> &Publisher {
        &self.publisher
    }

    pub fn put<M: AsRef<[u8]>>(&self, message: M) -> Result<(), ZenohError> {
        message
            .try_into_zbytes()
            .and_then(|p| self.publisher.put(p, None))
    }
}

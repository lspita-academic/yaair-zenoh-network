use std::sync::Arc;

use thiserror::Error;
use yaair::yaair::messages::serializer::Serializer;
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

use crate::{
    NetworkContext,
    message::{Message, MessagePacket},
};

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
        Ok(Self { _subscriber: subscriber })
    }

    unsafe extern "C" fn on_message<S: Serializer>(
        sample: *const <Sample as ZValue>::Value,
        context: *const NetworkContext<S>,
    ) {
        log::info!("Received message");
        let sample = Sample::zclone(sample);
        let context = unsafe { &*context };

        let payload_bytes = sample.payload().owned_bytes();
        let MessagePacket { sender, message } = match context.serializer.deserialize(&payload_bytes)
        {
            Ok(p) => p,
            Err(e) => {
                log::warn!("Failed to deserialize message packet: {e}");
                return;
            }
        };

        log::info!("Message sender: {sender}");
        log::debug!("Message: {message:?}");
        match context.messages.store(sender, message) {
            Ok(_) => log::info!("Message stored successfully"),
            Err(e) => log::warn!("Failed to store message: {e}"),
        }
    }
}

pub struct MessagePublisher {
    zid: ZId,
    publisher: Publisher,
}

#[derive(Debug, Error)]
pub enum PutError<SerializationError> {
    #[error("serialization error while trying to publish: {0}")]
    Serialization(SerializationError),
    #[error("zenoh error while trying to publish: {0}")]
    Zenoh(ZenohError),
}

impl MessagePublisher {
    pub fn new(session: &Session, base_keyexpr: &KeyExpr) -> ZenohResult<Self> {
        let zid = session.zid();
        let publisher = session.declare_publisher(
            &base_keyexpr.join_autocanonize(&KeyExpr::new(&zid.to_string())?)?,
            None,
        )?;
        Ok(Self { zid, publisher })
    }

    pub fn publisher(&self) -> &Publisher {
        &self.publisher
    }

    pub fn put<S: Serializer>(
        &self,
        message: Message,
        serializer: &S,
    ) -> Result<(), PutError<S::Error>> {
        let packet = MessagePacket::new(message, self.zid);
        let payload = serializer
            .serialize(&packet)
            .map_err(PutError::Serialization)
            .and_then(|v| v.try_into_zbytes().map_err(PutError::Zenoh))?;
        self.publisher.put(payload, None).map_err(PutError::Zenoh)
    }
}

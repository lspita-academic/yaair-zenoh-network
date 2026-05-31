use std::sync::Arc;

use serde::Deserialize;
use yaair::yaair::messages::serializer::Serializer;
pub use zenoh_pico::{
    keyexpr::KeyExpr,
    session::{
        Session,
        pubsub::{Publisher, Subscriber},
    },
};
use zenoh_pico::{
    result::ZenohError,
    sample::{Sample, SampleClosure},
    zbytes::TryIntoZBytes,
    zid::ZId,
    zvalue::{ZClone, ZClosure, ZValue},
};

use crate::comm::{
    CommunicationLayer, MessagePublisher, MessageSubscriber, MessageSubscriberOptions, TopicKeyExpr,
};

impl CommunicationLayer for Session {
    type Id = ZId;
    type Err = ZenohError;
    type KeyExpr = KeyExpr;

    fn node_id(&self) -> Self::Id {
        self.zid()
    }
}

impl TopicKeyExpr<Session> for KeyExpr {
    fn declare(keyexpr: &str) -> Result<Self, <Session as CommunicationLayer>::Err> {
        KeyExpr::autocanonize(keyexpr)
    }

    fn join(&self, keyexpr: &Self) -> Result<Self, <Session as CommunicationLayer>::Err> {
        self.join_autocanonize(keyexpr)
    }
}

unsafe extern "C" fn on_message<T: for<'de> Deserialize<'de>, Ser: Serializer>(
    sample: *const <Sample as ZValue>::Value,
    options: *const MessageSubscriberOptions<T, Ser>,
) {
    log::info!("Received message");
    let sample = Sample::zclone(sample);
    let options = unsafe { &*options };

    let payload_bytes = sample.payload().owned_bytes();
    log::debug!("Payload size: {}", payload_bytes.len());
    let message: T = match options.context.serializer.deserialize(&payload_bytes) {
        Ok(p) => p,
        Err(e) => {
            log::warn!("Failed to deserialize message packet: {e}");
            return;
        }
    };
    (options.callback)(message, &options.context);
}

impl MessageSubscriber<Session> for Subscriber {
    fn try_declare<T: for<'de> Deserialize<'de>, Ser: Serializer>(
        session: &Session,
        keyexpr: &<Session as CommunicationLayer>::KeyExpr,
        options: MessageSubscriberOptions<T, Ser>,
    ) -> Result<Self, <Session as CommunicationLayer>::Err> {
        let subscriber = session.declare_subscriber(
            &keyexpr,
            SampleClosure::from_callback(self::on_message::<T, Ser>, Some(Arc::new(options)))?,
            None,
        )?;
        Ok(subscriber)
    }

    fn listening_keyexpr(&self) -> &<Session as CommunicationLayer>::KeyExpr {
        self.keyexpr()
    }
}

impl MessagePublisher<Session> for Publisher {
    fn try_declare(
        session: &Session,
        keyexpr: &<Session as CommunicationLayer>::KeyExpr,
    ) -> Result<Self, <Session as CommunicationLayer>::Err> {
        let publisher = session.declare_publisher(&keyexpr, None)?;
        Ok(publisher)
    }

    fn put_message<M: AsRef<[u8]>>(
        &self,
        message: M,
    ) -> Result<(), <Session as CommunicationLayer>::Err> {
        message.try_into_zbytes().and_then(|p| self.put(p, None))
    }

    fn publishing_keyexpr(&self) -> &<Session as CommunicationLayer>::KeyExpr {
        self.keyexpr()
    }
}

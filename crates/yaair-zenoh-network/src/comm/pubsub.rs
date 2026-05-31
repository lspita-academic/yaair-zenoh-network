use std::sync::Arc;

use serde::Deserialize;
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
    zvalue::{ZClone, ZClosure, ZValue},
};

use crate::NetworkContext;

pub struct MessageSubscriber {
    _subscriber: Subscriber, // keep alive
}

pub struct MessageSubscriberOptions<T, S> {
    pub callback: fn(T, &NetworkContext<S>),
    pub context: Arc<NetworkContext<S>>,
}

impl MessageSubscriber {
    pub fn new<T: for<'de> Deserialize<'de>, S: Serializer>(
        session: &Session,
        base_keyexpr: &KeyExpr,
        context: MessageSubscriberOptions<T, S>,
    ) -> ZenohResult<Self> {
        let subscriber = session.declare_subscriber(
            &base_keyexpr.join_autocanonize(&KeyExpr::new("*")?)?,
            SampleClosure::from_callback(Self::on_message::<T, S>, Some(Arc::new(context)))?,
            None,
        )?;
        Ok(Self {
            _subscriber: subscriber,
        })
    }

    unsafe extern "C" fn on_message<T: for<'de> Deserialize<'de>, S: Serializer>(
        sample: *const <Sample as ZValue>::Value,
        options: *const MessageSubscriberOptions<T, S>,
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
}

pub struct MessagePublisher {
    publisher: Publisher,
}

impl MessagePublisher {
    pub fn new(session: &Session, keyexpr: &KeyExpr) -> ZenohResult<Self> {
        let publisher = session.declare_publisher(&keyexpr, None)?;
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

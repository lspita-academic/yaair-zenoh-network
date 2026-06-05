use std::sync::Arc;

use serde::Deserialize;
use yaair::yaair::messages::serializer::Serializer;
use zenoh_pico::{
    config::{Config, ConfigBuilder, ConfigMode},
    result::ZenohError,
    sample::{Sample, SampleClosure},
    zbytes::TryIntoZBytes,
    zid::ZId,
    zvalue::{ZClone, ZClosure, ZValue},
};
pub use zenoh_pico::{
    keyexpr::KeyExpr,
    session::{
        Session,
        pubsub::{Publisher, Subscriber},
    },
};

use crate::{
    ZenohNodeID,
    comm::{
        CommunicationLayer, MessagePublisher, MessageSubscriber, MessageSubscriberOptions,
        TopicKeyExpr,
    },
};

impl From<ZId> for ZenohNodeID {
    fn from(value: ZId) -> Self {
        Self::from(value.id)
    }
}

impl CommunicationLayer for Session {
    type Err = ZenohError;
    type Config = Config;
    type KeyExpr = KeyExpr;

    fn init(zenoh_config: Self::Config) -> Result<Self, Self::Err> {
        let mut config_builder = ConfigBuilder::default()
            .mode(ConfigMode::Peer)
            .scouting_timeout(zenoh_config.scouting_timeout);

        let mut multicast_locator = String::from("udp/224.0.0.224:7446");
        let mut listen_locator = String::from("udp/224.0.0.224:7447");
        if let Some(interface) = zenoh_config.interface {
            multicast_locator.push_str(&format!("#iface={interface}"));
            listen_locator.push_str(&format!("#iface={interface}"));
        }
        config_builder = config_builder.multicast_locator(&multicast_locator);
        config_builder = config_builder.listen(&listen_locator);

        if let Some(id) = zenoh_config.id {
            config_builder = config_builder.session_zid(id.into_bytes().into());
        }

        config_builder.build().and_then(|c| Self::open(c, None))
    }

    fn node_id(&self) -> ZenohNodeID {
        self.zid().into()
    }
}

impl TopicKeyExpr<Session> for KeyExpr {
    fn declare_topic(topic: &str) -> Result<Self, <Session as CommunicationLayer>::Err> {
        Self::autocanonize(topic)
    }

    fn join_topics(&self, other: &Self) -> Result<Self, <Session as CommunicationLayer>::Err> {
        self.join_autocanonize(other)
    }
}

unsafe extern "C" fn on_message<T: for<'de> Deserialize<'de>, Ser: Serializer + Sync + Send>(
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
    fn try_declare_background<
        T: for<'de> Deserialize<'de> + 'static,
        Ser: Serializer + Sync + Send + 'static,
    >(
        session: &Session,
        keyexpr: <Session as CommunicationLayer>::KeyExpr,
        options: MessageSubscriberOptions<T, Ser>,
    ) -> Result<Self, <Session as CommunicationLayer>::Err> {
        session.declare_subscriber(
            &keyexpr,
            SampleClosure::from_callback(self::on_message::<T, Ser>, Some(Arc::new(options)))?,
            None,
        )
    }
}

impl MessagePublisher<Session> for Publisher {
    fn try_declare(
        session: &Session,
        keyexpr: <Session as CommunicationLayer>::KeyExpr,
    ) -> Result<Self, <Session as CommunicationLayer>::Err> {
        session.declare_publisher(&keyexpr, None)
    }

    fn put_message<M: AsRef<[u8]>>(
        &self,
        message: M,
    ) -> Result<(), <Session as CommunicationLayer>::Err> {
        message.try_into_zbytes().and_then(|p| self.put(p, None))
    }
}

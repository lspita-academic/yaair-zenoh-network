use serde::Deserialize;
use yaair::yaair::messages::serializer::Serializer;
use zenoh::{
    Config, Error, Wait,
    config::ZenohId,
    pubsub::{Publisher as ZenohPublisher, Subscriber as ZenohSubscriber},
    sample::Sample,
};
pub use zenoh::{Session, key_expr::OwnedKeyExpr as KeyExpr};

use crate::comm::{
    CommunicationLayer, MessagePublisher, MessageSubscriber, MessageSubscriberOptions,
    TopicKeyExpr, ZenohConfig,
};

pub type Publisher = ZenohPublisher<'static>;
pub type Subscriber = ZenohSubscriber<()>;

impl CommunicationLayer for Session {
    type Id = ZenohId;
    type Err = Error;
    type KeyExpr = KeyExpr;

    fn init(zenoh_config: ZenohConfig<Self::Id>) -> Result<Self, Self::Err> {
        let mut config = Config::default();
        config.insert_json5("mode", "peer")?;
        config.insert_json5(
            "scouting/timeout",
            &zenoh_config.scouting_timeout.as_millis().to_string(),
        )?;
        if let Some(locator) = zenoh_config.multicast_locator {
            config.insert_json5("scouting/multicast/address", &locator)?;
        }
        if let Some(locator) = zenoh_config.listen_locator {
            config.insert_json5("listen/endpoints", &format!("[\"{locator}\"]"))?;
        }
        if let Some(id) = zenoh_config.id {
            config.insert_json5("id", &id.to_string())?;
        }
        zenoh::open(config).wait()
    }

    fn node_id(&self) -> Self::Id {
        self.zid()
    }
}

impl TopicKeyExpr<Session> for KeyExpr {
    fn declare_topic(topic: &str) -> Result<Self, <Session as CommunicationLayer>::Err> {
        topic.parse()
    }

    fn join_topics(&self, other: &Self) -> Result<Self, <Session as CommunicationLayer>::Err> {
        self.join(other)
    }
}

fn on_message<T: for<'de> Deserialize<'de>, Ser: Serializer + Sync + Send>(
    sample: Sample,
    options: &MessageSubscriberOptions<T, Ser>,
) {
    log::info!("Received message");

    let payload_bytes = sample.payload().to_bytes();
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
        session
            .declare_subscriber(keyexpr)
            .callback(move |sample| self::on_message(sample, &options))
            .wait()
    }
}

impl MessagePublisher<Session> for Publisher {
    fn try_declare(
        session: &Session,
        keyexpr: <Session as CommunicationLayer>::KeyExpr,
    ) -> Result<Self, <Session as CommunicationLayer>::Err> {
        session.declare_publisher(keyexpr).wait()
    }

    fn put_message<M: AsRef<[u8]>>(
        &self,
        message: M,
    ) -> Result<(), <Session as CommunicationLayer>::Err> {
        self.put(message.as_ref()).wait()
    }
}

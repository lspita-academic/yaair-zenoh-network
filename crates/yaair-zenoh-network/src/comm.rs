use std::sync::Arc;

use serde::Deserialize;
use yaair::yaair::messages::serializer::Serializer;

use crate::{NetworkContext, ZenohNodeId};

pub trait ZenohSession: Sized {
    type Err;
    type Config;
    type KeyExpr: ZenohKeyExpr<Self>;
    type Subscriber: ZenohSubscriber<Self>;
    type Publisher: ZenohPublisher<Self>;

    fn init(zenoh_config: Self::Config) -> Result<Self, Self::Err>;
    fn node_id(&self) -> ZenohNodeId;
    fn declare_subscriber<
        T: for<'de> Deserialize<'de> + 'static,
        Ser: Serializer + Sync + Send + 'static,
    >(
        &self,
        keyexpr: Self::KeyExpr,
        options: ZenohSubscriberOptions<T, Ser>,
    ) -> Result<Self::Subscriber, Self::Err>;
    fn declare_publisher(&self, keyexpr: Self::KeyExpr) -> Result<Self::Publisher, Self::Err>;
}

pub trait ZenohKeyExpr<Session: ZenohSession>: Sized {
    fn declare_topic(topic: &str) -> Result<Self, Session::Err>;
    fn join_topics(&self, other: &Self) -> Result<Self, Session::Err>;

    fn declare_join(&self, topic: &str) -> Result<Self, Session::Err> {
        Self::declare_topic(topic).and_then(|k| self.join_topics(&k))
    }

    fn star(&self) -> Result<Self, Session::Err> {
        self.declare_join("*")
    }
}

pub struct ZenohSubscriberOptions<T, Ser: Serializer + Sync + Send> {
    pub callback: fn(T, &NetworkContext<Ser>),
    pub context: Arc<NetworkContext<Ser>>,
}

pub trait ZenohSubscriber<Session: ZenohSession>: Sized {}

pub trait ZenohPublisher<Session: ZenohSession>: Sized {
    fn put_message<M: AsRef<[u8]>>(&self, message: M) -> Result<(), Session::Err>;
}

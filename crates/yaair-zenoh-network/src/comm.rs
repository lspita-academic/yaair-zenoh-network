use std::sync::Arc;

use serde::Deserialize;
use yaair::yaair::messages::serializer::Serializer;

use crate::{NetworkContext, ZenohNodeId};

pub trait ZenohSession: Sized {
    type Err;
    type Config;
    type KeyExpr: ZenohKeyExpr<Self>;

    fn init(zenoh_config: Self::Config) -> Result<Self, Self::Err>;
    fn node_id(&self) -> ZenohNodeId;
}

pub trait ZenohKeyExpr<Comm: ZenohSession>: Sized {
    fn declare_topic(topic: &str) -> Result<Self, Comm::Err>;
    fn join_topics(&self, other: &Self) -> Result<Self, Comm::Err>;

    fn declare_join(&self, topic: &str) -> Result<Self, Comm::Err> {
        Self::declare_topic(topic).and_then(|k| self.join_topics(&k))
    }

    fn star(&self) -> Result<Self, Comm::Err> {
        self.declare_join("*")
    }
}

pub struct ZenohSubscriberOptions<T, Ser: Serializer + Sync + Send> {
    pub callback: fn(T, &NetworkContext<Ser>),
    pub context: Arc<NetworkContext<Ser>>,
}

pub trait ZenohSubscriber<Comm: ZenohSession>: Sized {
    fn try_declare_background<
        T: for<'de> Deserialize<'de> + 'static,
        Ser: Serializer + Sync + Send + 'static,
    >(
        session: &Comm,
        keyexpr: Comm::KeyExpr,
        options: ZenohSubscriberOptions<T, Ser>,
    ) -> Result<Self, Comm::Err>;
}

pub trait ZenohPublisher<Comm: ZenohSession>: Sized {
    fn try_declare(session: &Comm, keyexpr: Comm::KeyExpr) -> Result<Self, Comm::Err>;
    fn put_message<M: AsRef<[u8]>>(&self, message: M) -> Result<(), Comm::Err>;
}

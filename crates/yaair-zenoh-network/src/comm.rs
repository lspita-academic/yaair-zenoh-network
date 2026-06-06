use std::{fmt::Display, hash::Hash, sync::Arc};

use serde::{Deserialize, Serialize};
use yaair::yaair::messages::serializer::Serializer;

use crate::NetworkContext;

pub type ZenohNodeIDBytes = [u8; 16];

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ZenohNodeId(ZenohNodeIDBytes);

impl ZenohNodeId {
    pub fn as_bytes(&self) -> &ZenohNodeIDBytes {
        &self.0
    }

    pub fn into_bytes(self) -> ZenohNodeIDBytes {
        self.0
    }
}

#[cfg_attr(zenoh_impl = "zenoh_full", allow(dead_code))]
pub(crate) trait FromZenohNodeId {
    fn from_node_id(node_id: ZenohNodeId) -> Self;
}

#[cfg_attr(zenoh_impl = "zenoh_full", allow(dead_code))]
pub(crate) trait IntoZenohNodeId {
    fn into_node_id(self) -> ZenohNodeId;
}

impl From<ZenohNodeIDBytes> for ZenohNodeId {
    fn from(value: ZenohNodeIDBytes) -> Self {
        Self(value)
    }
}

impl Display for ZenohNodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        hex::encode(&self.0).fmt(f)
    }
}

pub trait CommunicationLayer: Sized {
    type Err;
    type Config;
    type KeyExpr: TopicKeyExpr<Self>;

    fn init(zenoh_config: Self::Config) -> Result<Self, Self::Err>;
    fn node_id(&self) -> ZenohNodeId;
}

pub trait TopicKeyExpr<Comm: CommunicationLayer>: Sized {
    fn declare_topic(topic: &str) -> Result<Self, Comm::Err>;
    fn join_topics(&self, other: &Self) -> Result<Self, Comm::Err>;

    fn declare_join(&self, topic: &str) -> Result<Self, Comm::Err> {
        Self::declare_topic(topic).and_then(|k| self.join_topics(&k))
    }

    fn star(&self) -> Result<Self, Comm::Err> {
        self.declare_join("*")
    }
}

pub struct MessageSubscriberOptions<T, Ser: Serializer + Sync + Send> {
    pub callback: fn(T, &NetworkContext<Ser>),
    pub context: Arc<NetworkContext<Ser>>,
}

pub trait MessageSubscriber<Comm: CommunicationLayer>: Sized {
    fn try_declare_background<
        T: for<'de> Deserialize<'de> + 'static,
        Ser: Serializer + Sync + Send + 'static,
    >(
        session: &Comm,
        keyexpr: Comm::KeyExpr,
        options: MessageSubscriberOptions<T, Ser>,
    ) -> Result<Self, Comm::Err>;
}

pub trait MessagePublisher<Comm: CommunicationLayer>: Sized {
    fn try_declare(session: &Comm, keyexpr: Comm::KeyExpr) -> Result<Self, Comm::Err>;
    fn put_message<M: AsRef<[u8]>>(&self, message: M) -> Result<(), Comm::Err>;
}

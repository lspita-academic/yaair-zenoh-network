#[cfg(zenoh_impl = "zenoh_full")]
#[path = "zenoh_full.rs"]
pub mod zenoh;

#[cfg(zenoh_impl = "zenoh_pico")]
#[path = "zenoh_pico.rs"]
pub mod zenoh;

use std::{fmt::Display, hash::Hash, sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};
use yaair::yaair::messages::serializer::Serializer;

use crate::NetworkContext;

pub type ZenohNodeIDBytes = [u8; 16];

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ZenohNodeID(ZenohNodeIDBytes);

impl ZenohNodeID {
    pub fn as_bytes(&self) -> &ZenohNodeIDBytes {
        &self.0
    }

    pub fn into_bytes(self) -> ZenohNodeIDBytes {
        self.0
    }
}

impl From<ZenohNodeIDBytes> for ZenohNodeID {
    fn from(value: ZenohNodeIDBytes) -> Self {
        Self(value)
    }
}

impl Display for ZenohNodeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        hex::encode(&self.0).fmt(f)
    }
}

pub struct ZenohConfig {
    pub scouting_timeout: Duration,
    pub interface: Option<String>,
    pub id: Option<ZenohNodeID>,
}

impl Default for ZenohConfig {
    fn default() -> Self {
        Self {
            scouting_timeout: Duration::from_secs(30),
            interface: Default::default(),
            id: Default::default(),
        }
    }
}

pub trait CommunicationLayer: Sized {
    type Err;
    type KeyExpr: TopicKeyExpr<Self>;

    fn init(zenoh_config: ZenohConfig) -> Result<Self, Self::Err>;
    fn node_id(&self) -> ZenohNodeID;
}

pub trait TopicKeyExpr<Comm: CommunicationLayer>: Sized {
    fn declare_topic(topic: &str) -> Result<Self, Comm::Err>;
    fn join_topics(&self, other: &Self) -> Result<Self, Comm::Err>;

    fn declare_join(&self, topic: &str) -> Result<Self, Comm::Err> {
        Self::declare_topic(topic).and_then(|k| self.join_topics(&k))
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

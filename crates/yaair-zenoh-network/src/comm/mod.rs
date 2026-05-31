mod zenoh_pico;
pub mod zenoh {
    #[cfg(zenoh_impl = "zenoh_full")]
    pub use super::zenoh_full::{KeyExpr, Publisher, Session, Subscriber};
    #[cfg(zenoh_impl = "zenoh_pico")]
    pub use super::zenoh_pico::{KeyExpr, Publisher, Session, Subscriber};
}

use std::{fmt::Display, hash::Hash, sync::Arc};

use serde::Deserialize;
use yaair::yaair::messages::serializer::Serializer;

use crate::NetworkContext;

pub trait TopicKeyExpr<Comm: CommunicationLayer>: Sized {
    fn declare(keyexpr: &str) -> Result<Self, Comm::Err>;
    fn join(&self, keyexpr: &Self) -> Result<Self, Comm::Err>;

    fn declare_join(&self, keyexpr: &str) -> Result<Self, Comm::Err> {
        self.join(&Self::declare(keyexpr)?)
    }
}

pub trait CommunicationLayer: Sized {
    type Id: Display + Ord + Hash + Copy;
    type Err;
    type KeyExpr: TopicKeyExpr<Self>;

    fn node_id(&self) -> Self::Id;
}

pub struct MessageSubscriberOptions<T, Ser> {
    pub callback: fn(T, &NetworkContext<Ser>),
    pub context: Arc<NetworkContext<Ser>>,
}

pub trait MessageSubscriber<Comm: CommunicationLayer>: Sized {
    fn try_declare<T: for<'de> Deserialize<'de>, Ser: Serializer>(
        session: &Comm,
        keyexpr: &Comm::KeyExpr,
        options: MessageSubscriberOptions<T, Ser>,
    ) -> Result<Self, Comm::Err>;

    #[allow(dead_code, reason = "for simmetry with publisher")]
    fn listening_keyexpr(&self) -> &Comm::KeyExpr;
}

pub trait MessagePublisher<Comm: CommunicationLayer>: Sized {
    fn try_declare(session: &Comm, keyexpr: &Comm::KeyExpr) -> Result<Self, Comm::Err>;
    fn put_message<M: AsRef<[u8]>>(&self, message: M) -> Result<(), Comm::Err>;
    fn publishing_keyexpr(&self) -> &Comm::KeyExpr;
}

//! Heartbit communication for [`ZenohNetwork`](crate::ZenohNetwork) nodes to
//! notify their existance.
//!
//! This module is provided under the [`heartbit`](crate#features) feature.

use std::{sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};
use yaair::yaair::messages::serializer::Serializer;

use crate::{
    NetworkContext, ZenohNodeId,
    comm::{ZenohSession, ZenohPublisher},
    zenoh_impl::comm::{KeyExpr, Publisher, Session},
};

#[derive(Serialize, Deserialize)]
pub(crate) struct Heartbit {
    pub sender: ZenohNodeId,
    pub lifespan: Option<Duration>,
}

/// A publisher for heartbit messages.
///
/// It can be created by [declaring it from a
/// `ZenohNetwork`](crate::ZenohNetwork::declare_heartbit_publisher).
pub struct HeartbitPublisher<Ser: Serializer + Sync + Send> {
    node_id: ZenohNodeId,
    network_context: Arc<NetworkContext<Ser>>,
    publisher: Publisher,
}

impl<'a, Ser: Serializer + Sync + Send> HeartbitPublisher<Ser> {
    pub(crate) fn try_declare(
        session: &Session,
        keyexpr: KeyExpr,
        network_context: Arc<NetworkContext<Ser>>,
    ) -> Result<Self, <Session as ZenohSession>::Err> {
        let node_id = session.node_id();
        let publisher = Publisher::try_declare(session, keyexpr)?;
        Ok(Self {
            node_id,
            network_context,
            publisher,
        })
    }

    /// Publish a keepalive message for the node.
    ///
    /// This allows other nodes to know the caller is alive and update the
    /// timestamp of the last message to reset the countdown for considering the
    /// node offline.
    pub fn put_keep_alive(&self) {
        self.put_heartbit(self.heartbit(None));
    }

    /// Publish a new lifespan to use for the node.
    ///
    /// This allows other nodes to know the caller should be considered alive
    /// for a different lifespan than [the default of the
    /// network](crate::config::ZenohNetworkConfig::lifespan).
    pub fn put_lifespan(&self, lifespan: Duration) {
        self.put_heartbit(self.heartbit(Some(lifespan)));
    }

    fn heartbit(&self, lifespan: Option<Duration>) -> Heartbit {
        Heartbit {
            sender: self.node_id,
            lifespan,
        }
    }

    fn put_heartbit(&self, heartbit: Heartbit) {
        log::info!("Preparing heartbit message");
        let heartbit_bytes = match self.network_context.serializer.serialize(&heartbit) {
            Ok(v) => v,
            Err(e) => {
                log::warn!("Failed to serialize heartbit: {e}");
                return;
            }
        };
        match self.publisher.put_message(heartbit_bytes) {
            Ok(_) => log::info!("Heartbit published successfully"),
            Err(e) => log::info!("Error publishing heartbit: {e}"),
        }
    }
}

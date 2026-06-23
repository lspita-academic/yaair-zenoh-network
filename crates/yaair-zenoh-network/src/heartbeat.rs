//! Heartbeat communication for [`ZenohNetwork`](crate::ZenohNetwork) nodes to
//! notify their existance.
//!
//! This module is provided under the [`heartbeat`](crate#features) feature.

use std::{sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};
use yaair::yaair::messages::serializer::Serializer;

use crate::{
    NetworkContext, ZenohNodeId,
    comm::{ZenohPublisher, ZenohSession},
    zenoh_impl::comm::{KeyExpr, Publisher, Session},
};

#[derive(Serialize, Deserialize)]
pub(crate) struct Heartbeat {
    pub sender: ZenohNodeId,
    pub lifespan: Option<Duration>,
}

/// A publisher for heartbeat messages.
///
/// It can be created by [declaring it from a
/// `ZenohNetwork`](crate::ZenohNetwork::declare_heartbeat_publisher).
pub struct HeartbeatPublisher<Ser: Serializer + Sync + Send> {
    node_id: ZenohNodeId,
    network_context: Arc<NetworkContext<Ser>>,
    publisher: Publisher,
}

impl<'a, Ser: Serializer + Sync + Send> HeartbeatPublisher<Ser> {
    pub(crate) fn try_declare(
        session: &Session,
        keyexpr: KeyExpr,
        network_context: Arc<NetworkContext<Ser>>,
    ) -> Result<Self, <Session as ZenohSession>::Err> {
        let node_id = session.node_id();
        let publisher = <Session as ZenohSession>::declare_publisher(session, keyexpr)?;
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
        self.put_heartbeat(self.heartbeat(None));
    }

    /// Publish a new lifespan to use for the node.
    ///
    /// This allows other nodes to know the caller should be considered alive
    /// for a different lifespan than [the default of the
    /// network](crate::config::ZenohNetworkConfig::lifespan).
    pub fn put_lifespan(&self, lifespan: Duration) {
        self.put_heartbeat(self.heartbeat(Some(lifespan)));
    }

    fn heartbeat(&self, lifespan: Option<Duration>) -> Heartbeat {
        Heartbeat {
            sender: self.node_id,
            lifespan,
        }
    }

    fn put_heartbeat(&self, heartbeat: Heartbeat) {
        log::info!("Preparing heartbeat message");
        let heartbeat_bytes = match self.network_context.serializer.serialize(&heartbeat) {
            Ok(v) => v,
            Err(e) => {
                log::warn!("Failed to serialize heartbeat: {e}");
                return;
            }
        };
        match self.publisher.put_message(heartbeat_bytes) {
            Ok(_) => log::info!("Heartbeat published successfully"),
            Err(e) => log::info!("Error publishing heartbeat: {e}"),
        }
    }
}

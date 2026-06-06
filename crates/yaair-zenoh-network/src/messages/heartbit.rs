use std::time::Duration;

use serde::{Deserialize, Serialize};
use yaair::yaair::messages::serializer::Serializer;

use crate::{
    ZenohNodeId,
    comm::{CommunicationLayer, MessagePublisher},
    zenoh_impl::comm::{KeyExpr, Publisher, Session},
};

#[derive(Serialize, Deserialize)]
pub struct Heartbit {
    pub sender: ZenohNodeId,
    pub lifespan: Option<Duration>,
}

pub struct HeartbitPublisher<Ser> {
    node_id: ZenohNodeId,
    serializer: Ser,
    publisher: Publisher,
}

impl<Ser: Serializer> HeartbitPublisher<Ser> {
    pub(crate) fn try_declare(
        session: &Session,
        keyexpr: KeyExpr,
        serializer: Ser,
    ) -> Result<Self, <Session as CommunicationLayer>::Err> {
        let node_id = session.node_id();
        let publisher = Publisher::try_declare(session, keyexpr)?;
        Ok(Self {
            node_id,
            serializer,
            publisher,
        })
    }

    pub fn put_keep_alive(&self) {
        self.put_heartbit(self.heartbit(None));
    }

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
        let heartbit_bytes = match self.serializer.serialize(&heartbit) {
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

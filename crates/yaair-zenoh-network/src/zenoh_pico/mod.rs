pub mod comm;
pub mod config;

pub use zenoh_pico::result::ZenohError;
use zenoh_pico::zid::ZId;

use crate::{
    ZenohNodeId,
    comm::{FromZenohNodeId, IntoZenohNodeId},
};

impl FromZenohNodeId for ZId {
    fn from_node_id(node_id: ZenohNodeId) -> Self {
        Self::from(node_id.into_bytes())
    }
}

impl IntoZenohNodeId for ZId {
    fn into_node_id(self) -> ZenohNodeId {
        self.id.into()
    }
}

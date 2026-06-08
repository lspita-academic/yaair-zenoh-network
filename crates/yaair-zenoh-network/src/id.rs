//! Primitives related to what the [`ZenohNetwork`](crate::ZenohNetwork) uses to
//! identify nodes.

use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// An array representing the underlying data of a [`ZenohNodeId`].
pub type ZenohNodeIDBytes = [u8; 16];

/// A unique identifier for a node in the [`ZenohNetwork`](crate::ZenohNetwork).
#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ZenohNodeId(ZenohNodeIDBytes);

impl ZenohNodeId {
    /// Returns a reference to the underlying bytes.
    pub fn as_bytes(&self) -> &ZenohNodeIDBytes {
        &self.0
    }

    /// Converts [`self`](Self) into the underlying bytes.
    pub fn into_bytes(self) -> ZenohNodeIDBytes {
        self.0
    }
}

// custom from/into traits are needed to avoid potential conflicts errors
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

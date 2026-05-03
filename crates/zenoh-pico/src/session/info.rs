use std::sync::Arc;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

use crate::zid::ZId;

pub struct PeersInfo {
    pub(super) signal: Arc<Signal<CriticalSectionRawMutex, ZId>>,
}

impl PeersInfo {
    pub async fn recv_async(&self) -> ZId {
        self.signal.wait().await
    }
}

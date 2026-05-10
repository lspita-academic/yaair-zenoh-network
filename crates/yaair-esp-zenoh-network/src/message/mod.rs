pub mod pubsub;
pub mod store;

use serde::{Deserialize, Serialize};
use zenoh_pico::zid::ZId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    payload: Vec<u8>,
}

impl Message {
    pub fn new(payload: Vec<u8>) -> Self {
        Self { payload }
    }
}

impl Into<Vec<u8>> for Message {
    fn into(self) -> Vec<u8> {
        self.payload
    }
}

/// NOTE: the zenoh sample source info api is marked as unstable, so instead we
/// send the session zid in the payload to recognize peers.
#[derive(Serialize, Deserialize, Clone)]
pub struct MessagePacket {
    message: Message,
    sender: ZId,
}

impl MessagePacket {
    pub fn new(message: Message, sender: ZId) -> Self {
        Self { message, sender }
    }
}

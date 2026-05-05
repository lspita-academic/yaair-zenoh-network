use std::{collections::HashMap, sync::Mutex, time::SystemTime};

use serde::{Deserialize, Serialize};
use yaair::yaair::messages::serializer::Serializer;
use zenoh_pico::zid::ZId;

use crate::atomic::{
    AtomicLockResultExtensions, AtomicResult, AtomicResultExtensions, PoisonedLockError,
};

#[derive(Serialize, Deserialize)]
pub struct MessagePacket {
    payload: Vec<u8>,
    sender: ZId,
}

impl MessagePacket {
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn sender(&self) -> ZId {
        self.sender
    }
}

pub struct Message {
    payload: Vec<u8>,
    timestamp: SystemTime,
}

impl Message {
    pub fn new(payload: Vec<u8>) -> Self {
        let timestamp = SystemTime::now();
        Self { payload, timestamp }
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }
}

impl From<&MessagePacket> for Message {
    fn from(value: &MessagePacket) -> Self {
        Message::new(value.payload().to_owned())
    }
}

impl From<MessagePacket> for Message {
    fn from(value: MessagePacket) -> Self {
        Message::new(value.payload)
    }
}

pub struct AtomicMessagesStore {
    storage: Mutex<HashMap<ZId, Message>>,
}

impl AtomicMessagesStore {
    pub fn new() -> Self {
        Self {
            storage: Mutex::new(HashMap::new()),
        }
    }

    pub fn store(&self, zid: ZId, message: Message) -> Result<(), PoisonedLockError> {
        let mut storage = self.storage.lock().map_err(|_| PoisonedLockError)?;
        storage.insert(zid, message);
        Ok(())
    }
}

pub struct AtomicMessageSerializer<S> {
    serializer: Mutex<S>,
}

impl<S: Serializer> AtomicMessageSerializer<S> {
    pub fn new(serializer: S) -> Self {
        Self {
            serializer: Mutex::new(serializer),
        }
    }

    pub fn deserialize_packet<P: AsRef<[u8]>>(
        &self,
        payload: P,
    ) -> AtomicResult<MessagePacket, S::Error> {
        let serializer = self.serializer.lock().atomic_lock()?;
        serializer.deserialize(payload.as_ref()).atomic()
    }

    pub fn serialize_payload<T: Serialize>(&self, value: &T) -> AtomicResult<Vec<u8>, S::Error> {
        let serializer = self.serializer.lock().atomic_lock()?;
        serializer.serialize(value).atomic()
    }
}

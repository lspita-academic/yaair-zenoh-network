use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard},
    time::{Duration, SystemTime},
};

use thiserror::Error;
use zenoh_pico::zid::ZId;

use crate::message::Message;

#[derive(Clone)]
pub struct StoreMessage {
    message: Message,
    timestamp: SystemTime,
}

impl StoreMessage {
    pub fn new(message: Message) -> Self {
        let timestamp = SystemTime::now();
        Self { message, timestamp }
    }

    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }
}

impl From<Message> for StoreMessage {
    fn from(value: Message) -> Self {
        StoreMessage::new(value)
    }
}

impl Into<Message> for StoreMessage {
    fn into(self) -> Message {
        self.message
    }
}

type Storage = HashMap<ZId, StoreMessage>;

pub struct AtomicMessagesStore {
    lifespan: Duration,
    storage: Mutex<Storage>,
}

#[derive(Debug, Error)]
#[error("poisoned lock")]
pub struct PoisonedLockError;

impl AtomicMessagesStore {
    pub fn new(lifespan: Duration) -> Self {
        Self {
            lifespan,
            storage: Default::default(),
        }
    }

    pub fn acquire_storage(&self) -> Result<MutexGuard<'_, Storage>, PoisonedLockError> {
        self.storage.lock().map_err(|_| PoisonedLockError)
    }

    pub fn store(&self, zid: ZId, message: Message) -> Result<(), PoisonedLockError> {
        let store_message = StoreMessage::new(message);
        self.acquire_storage()?.insert(zid, store_message);
        Ok(())
    }

    pub fn clear_dead(&self) -> Result<(), PoisonedLockError> {
        let mut storage = self.acquire_storage()?;
        let expired_keys: Vec<_> = storage
            .iter()
            .map(|(key, m)| (key, m.timestamp()))
            .filter_map(|(key, t)| {
                t.elapsed().ok().and_then(|e| {
                    if e >= self.lifespan {
                        Some(key.clone())
                    } else {
                        None
                    }
                })
            })
            .collect();
        expired_keys.into_iter().for_each(|k| {
            storage.remove(&k);
        });
        Ok(())
    }

    pub fn snapshot(&self) -> Result<Storage, PoisonedLockError> {
        self.acquire_storage().map(|s| s.clone())
    }
}

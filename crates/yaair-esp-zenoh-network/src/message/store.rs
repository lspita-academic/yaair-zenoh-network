use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard},
    time::{Duration, SystemTime},
};

use thiserror::Error;
use yaair::yaair::messages::valuetree::ValueTree;
use zenoh_pico::zid::ZId;

#[derive(Clone)]
pub struct StoreMessage {
    payload: ValueTree,
    timestamp: SystemTime,
}

impl StoreMessage {
    pub fn new(payload: ValueTree) -> Self {
        let timestamp = SystemTime::now();
        Self { payload, timestamp }
    }

    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }
}

impl From<ValueTree> for StoreMessage {
    fn from(value: ValueTree) -> Self {
        StoreMessage::new(value)
    }
}

impl Into<ValueTree> for StoreMessage {
    fn into(self) -> ValueTree {
        self.payload
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

    pub fn store(&self, zid: ZId, payload: ValueTree) -> Result<(), PoisonedLockError> {
        let store_message = StoreMessage::new(payload);
        self.acquire_storage()?.insert(zid, store_message);
        Ok(())
    }

    pub fn clear_dead(&self) -> Result<(), PoisonedLockError> {
        let mut storage = self.acquire_storage()?;
        let expired_keys: Vec<_> = storage
            .iter()
            .map(|(zid, m)| (zid, m.timestamp()))
            .filter_map(|(zid, t)| {
                t.elapsed().ok().and_then(|e| {
                    if e >= self.lifespan {
                        Some(zid.clone())
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

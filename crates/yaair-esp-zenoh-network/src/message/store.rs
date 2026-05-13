use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard},
    time::{Duration, SystemTime},
};

use thiserror::Error;
use zenoh_pico::zid::ZId;

#[derive(Clone)]
pub struct StoreMessage<T> {
    payload: T,
    timestamp: SystemTime,
}

impl<T> StoreMessage<T> {
    pub fn new(payload: T) -> Self {
        let timestamp = SystemTime::now();
        Self { payload, timestamp }
    }

    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }

    pub fn into_inner(self) -> T {
        self.payload
    }
}

impl<T> From<T> for StoreMessage<T> {
    fn from(value: T) -> Self {
        StoreMessage::new(value)
    }
}

type Storage<T> = HashMap<ZId, StoreMessage<T>>;

pub struct AtomicMessagesStore<T> {
    lifespan: Duration,
    storage: Mutex<Storage<T>>,
}

#[derive(Debug, Error)]
#[error("poisoned lock")]
pub struct PoisonedLockError;

impl<T> AtomicMessagesStore<T> {
    pub fn new(lifespan: Duration) -> Self {
        Self {
            lifespan,
            storage: Default::default(),
        }
    }

    pub fn acquire_storage(&self) -> Result<MutexGuard<'_, Storage<T>>, PoisonedLockError> {
        self.storage.lock().map_err(|_| PoisonedLockError)
    }

    pub fn store(&self, zid: ZId, payload: T) -> Result<(), PoisonedLockError> {
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
}

impl<T: Clone> AtomicMessagesStore<T> {
    pub fn snapshot(&self) -> Result<Storage<T>, PoisonedLockError> {
        self.acquire_storage().map(|s| s.clone())
    }
}

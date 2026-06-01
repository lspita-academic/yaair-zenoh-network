use std::{
    collections::HashMap, hash::Hash, sync::{Mutex, MutexGuard}, time::{Duration, SystemTime}
};

use thiserror::Error;

#[derive(Clone)]
pub struct StoreEntity<T> {
    message: T,
    timestamp: SystemTime,
}

impl<T> StoreEntity<T> {
    pub fn new(message: T) -> Self {
        Self { message, timestamp: SystemTime::now() }
    }

    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }

    pub fn update_message(&mut self, message: T) {
        self.message = message;
        self.keep_alive();
    }

    pub fn keep_alive(&mut self) {
        self.timestamp = SystemTime::now();
    }

    pub fn into_inner(self) -> T {
        self.message
    }
}

impl<T> From<T> for StoreEntity<T> {
    fn from(value: T) -> Self {
        StoreEntity::new(value)
    }
}

type Storage<Id, T> = HashMap<Id, StoreEntity<T>>;

pub struct AtomicMessagesStore<Id, T> {
    lifespan: Duration,
    storage: Mutex<Storage<Id, T>>,
}

#[derive(Debug, Error)]
#[error("poisoned lock")]
pub struct PoisonedLockError;

impl<Id: Eq + Hash + Clone, T> AtomicMessagesStore<Id, T> {
    pub fn new(lifespan: Duration) -> Self {
        Self {
            lifespan,
            storage: Default::default(),
        }
    }

    pub fn acquire_storage(&self) -> Result<MutexGuard<'_, Storage<Id, T>>, PoisonedLockError> {
        self.storage.lock().map_err(|_| PoisonedLockError)
    }

    pub fn store(&self, id: Id, message: T) -> Result<(), PoisonedLockError> {
        let mut storage = self.acquire_storage()?;
        if let Some(entity) = storage.get_mut(&id) {
            entity.update_message(message);
        } else {
            storage.insert(id, StoreEntity::new(message));
        }
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

impl<Id: Eq + Hash + Clone, T: Clone> AtomicMessagesStore<Id, T> {
    pub fn snapshot(&self) -> Result<Storage<Id, T>, PoisonedLockError> {
        self.acquire_storage().map(|s| s.clone())
    }
}

use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Mutex, MutexGuard},
    time::{Duration, SystemTime},
};

use thiserror::Error;

#[derive(Clone)]
pub struct StoreEntity<T> {
    message: Option<T>,
    lifespan: Duration,
    timestamp: SystemTime,
}

impl<T> StoreEntity<T> {
    pub fn new(message: Option<T>, lifespan: Duration) -> Self {
        Self {
            message,
            lifespan,
            timestamp: SystemTime::now(),
        }
    }

    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }

    pub fn lifespan(&self) -> Duration {
        self.lifespan
    }

    pub fn message(&self) -> Option<&T> {
        self.message.as_ref()
    }

    pub fn update_message(&mut self, message: T) {
        self.message = Some(message);
        self.keep_alive();
    }

    pub fn update_lifespan(&mut self, lifespan: Duration) {
        self.lifespan = lifespan;
        self.keep_alive();
    }

    pub fn keep_alive(&mut self) {
        self.timestamp = SystemTime::now();
    }
}

type Map<K, T> = HashMap<K, T>;
type Storage<Id, T> = Map<Id, StoreEntity<T>>;

pub struct AtomicMessagesStore<Id, T> {
    default_lifespan: Duration,
    storage: Mutex<Storage<Id, T>>,
}

#[derive(Debug, Error)]
#[error("poisoned lock")]
pub struct PoisonedLockError;

impl<Id: Eq + Hash + Clone, T> AtomicMessagesStore<Id, T> {
    pub fn new(default_lifespan: Duration) -> Self {
        Self {
            default_lifespan,
            storage: Default::default(),
        }
    }

    pub fn acquire_storage(&self) -> Result<MutexGuard<'_, Storage<Id, T>>, PoisonedLockError> {
        self.storage.lock().map_err(|_| PoisonedLockError)
    }

    pub fn store_message(&self, id: Id, message: T) -> Result<(), PoisonedLockError> {
        let mut storage = self.acquire_storage()?;
        if let Some(entity) = storage.get_mut(&id) {
            entity.update_message(message);
        } else {
            storage.insert(id, StoreEntity::new(Some(message), self.default_lifespan));
        }
        Ok(())
    }

    pub fn store_lifespan(&self, id: Id, lifespan: Duration) -> Result<(), PoisonedLockError> {
        let mut storage = self.acquire_storage()?;
        if let Some(entity) = storage.get_mut(&id) {
            entity.update_lifespan(lifespan);
        } else {
            storage.insert(id, StoreEntity::new(None, lifespan));
        }
        Ok(())
    }

    pub fn keep_alive(&self, id: Id) -> Result<(), PoisonedLockError> {
        let mut storage = self.acquire_storage()?;
        if let Some(entity) = storage.get_mut(&id) {
            entity.keep_alive();
        } else {
            storage.insert(id, StoreEntity::new(None, self.default_lifespan));
        }
        Ok(())
    }

    pub fn clear_dead(&self) -> Result<(), PoisonedLockError> {
        let mut storage = self.acquire_storage()?;
        let expired_keys: Vec<_> = storage
            .iter()
            .filter_map(|(zid, e)| {
                e.timestamp().elapsed().ok().and_then(|elapsed| {
                    if elapsed >= e.lifespan() {
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
    pub fn messages_snapshot(&self) -> Result<Map<Id, T>, PoisonedLockError> {
        let storage = self.acquire_storage()?;
        Ok(storage
            .iter()
            .filter_map(|(id, e)| e.message().cloned().map(|m| (id.clone(), m)))
            .collect())
    }
}

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

    pub fn update_message(&mut self, message: T) -> Option<T> {
        self.keep_alive();
        Self::update(&mut self.message, Some(message))
    }

    #[cfg(feature = "heartbit")]
    pub fn update_lifespan(&mut self, lifespan: Duration) -> Duration {
        self.keep_alive();
        Self::update(&mut self.lifespan, lifespan)
    }

    pub fn keep_alive(&mut self) -> SystemTime {
        Self::update(&mut self.timestamp, SystemTime::now())
    }

    fn update<Value>(dest: &mut Value, src: Value) -> Value {
        let prev = std::mem::replace(dest, src);
        prev
    }
}

type Map<K, T> = HashMap<K, T>;
type Storage<Id, T> = Map<Id, StoreEntity<T>>;

pub struct MessagesStore<Id, T> {
    default_lifespan: Duration,
    storage: Mutex<Storage<Id, T>>,
}

#[derive(Debug, Error)]
#[error("poisoned lock")]
pub struct PoisonedLockError;

impl<Id: Eq + Hash + Clone, T> MessagesStore<Id, T> {
    pub fn new(default_lifespan: Duration) -> Self {
        Self {
            default_lifespan,
            storage: Default::default(),
        }
    }

    fn acquire_storage(&self) -> Result<MutexGuard<'_, Storage<Id, T>>, PoisonedLockError> {
        self.storage.lock().map_err(|_| PoisonedLockError)
    }

    pub fn store_message(&self, id: Id, message: T) -> Result<Option<T>, PoisonedLockError> {
        let mut storage = self.acquire_storage()?;
        // if-else instead of and_then/or_else to prevent cloning `message`
        Ok(if let Some(e) = storage.get_mut(&id) {
            e.update_message(message)
        } else {
            storage.insert(id, StoreEntity::new(Some(message), self.default_lifespan));
            None
        })
    }

    #[cfg(feature = "heartbit")]
    pub fn store_lifespan(
        &self,
        id: Id,
        lifespan: Duration,
    ) -> Result<Option<Duration>, PoisonedLockError> {
        let mut storage = self.acquire_storage()?;
        Ok(storage
            .get_mut(&id)
            .and_then(|e| Some(e.update_lifespan(lifespan)))
            .or_else(|| {
                storage.insert(id, StoreEntity::new(None, lifespan));
                None
            }))
    }

    #[cfg(feature = "heartbit")]
    pub fn keep_alive(&self, id: Id) -> Result<Option<SystemTime>, PoisonedLockError> {
        let mut storage = self.acquire_storage()?;
        Ok(storage
            .get_mut(&id)
            .and_then(|e| Some(e.keep_alive()))
            .or_else(|| {
                storage.insert(id, StoreEntity::new(None, self.default_lifespan));
                None
            }))
    }

    pub fn clear_dead(&self) -> Result<Vec<Id>, PoisonedLockError> {
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
        expired_keys.iter().for_each(|k| {
            storage.remove(k);
        });
        Ok(expired_keys)
    }
}

impl<Id: Eq + Hash + Clone, T: Clone> MessagesStore<Id, T> {
    pub fn create_snapshot(&self) -> Result<Map<Id, T>, PoisonedLockError> {
        let storage = self.acquire_storage()?;
        Ok(storage
            .iter()
            .filter_map(|(id, e)| e.message().cloned().map(|m| (id.clone(), m)))
            .collect())
    }
}

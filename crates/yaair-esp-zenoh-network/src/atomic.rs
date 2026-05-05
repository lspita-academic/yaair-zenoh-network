use std::sync::LockResult;

use thiserror::Error;

#[derive(Debug, Error)]
#[error("poisoned lock")]
pub struct PoisonedLockError;

#[derive(Debug, Error)]
pub enum AtomicError<E> {
    #[error(transparent)]
    PoisonedLock(PoisonedLockError),
    #[error(transparent)]
    Other(E),
}

pub type AtomicResult<T, E> = Result<T, AtomicError<E>>;

pub trait AtomicResultExtensions<T, E> {
    fn atomic(self) -> AtomicResult<T, E>;
}

impl<T, E> AtomicResultExtensions<T, E> for Result<T, E> {
    fn atomic(self) -> AtomicResult<T, E> {
        self.map_err(AtomicError::Other)
    }
}

pub trait AtomicLockResultExtensions<T, E> {
    fn atomic_lock(self) -> AtomicResult<T, E>;
}

impl<T, E> AtomicLockResultExtensions<T, E> for LockResult<T> {
    fn atomic_lock(self) -> AtomicResult<T, E> {
        self.map_err(|_| AtomicError::PoisonedLock(PoisonedLockError))
    }
}

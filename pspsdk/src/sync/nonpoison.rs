//! Non-poisoning synchronous locks.
//!
//! The difference from the locks in the [`poison`] module is that the locks in this module will not
//! become poisoned when a thread panics while holding a guard.
//!
//! [`poison`]: super::poison
use core::fmt;

mod mutex;
pub use mutex::{MappedMutexGuard, Mutex, MutexGuard};

mod reentrant_lock;
pub use reentrant_lock::{ReentrantLock, ReentrantLockGuard};

mod rwlock;
pub use rwlock::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};

pub use super::poison::Once;

mod once_lock;
pub use once_lock::OnceLock;

mod condvar;
pub use condvar::Condvar;

mod lazy_lock;
pub use lazy_lock::LazyLock;

mod barrier;
pub use barrier::{Barrier, BarrierWaitResult};

/// A type alias for the result of a nonblocking locking method.
pub type TryLockResult<Guard> = Result<Guard, WouldBlock>;

/// A lock could not be acquired at this time because the operation would otherwise block.
pub struct WouldBlock;

impl fmt::Debug for WouldBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "WouldBlock".fmt(f)
    }
}

impl fmt::Display for WouldBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "try_lock failed because the operation would block".fmt(f)
    }
}

impl core::error::Error for WouldBlock {}

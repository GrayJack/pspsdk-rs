use core::fmt;

mod mutex;
pub use mutex::{MappedMutexGuard, Mutex, MutexGuard};

mod reentrant_lock;
pub use reentrant_lock::{ReentrantLock, ReentrantLockGuard};

/// An enumeration of possible errors associated with a [`TryLockResult`] which
/// can occur while trying to acquire a lock, from the [`try_lock`] method on a
/// [`Mutex`] or the [`try_read`] and [`try_write`] methods on an [`RwLock`].
///
/// [`try_lock`]: crate::sync::nonpoison::Mutex::try_lock
/// [`try_read`]: crate::sync::nonpoison::RwLock::try_read
/// [`try_write`]: crate::sync::nonpoison::RwLock::try_write
/// [`Mutex`]: crate::sync::nonpoison::Mutex
/// [`RwLock`]: crate::sync::nonpoison::RwLock
pub enum TryLockError {
    /// The lock could not be acquired at this time because the operation would
    /// otherwise block.
    WouldBlock,
}

/// A type alias for the result of a nonblocking locking method.
///
/// For more information, see [`LockResult`]. A `TryLockResult` doesn't
/// necessarily hold the associated guard in the [`Err`] type as the lock might not
/// have been acquired for other reasons.
///
/// [`LockResult`]: crate::sync::poison::LockResult
pub type TryLockResult<Guard> = Result<Guard, TryLockError>;


impl fmt::Debug for TryLockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TryLockError::WouldBlock => "WouldBlock".fmt(f),
        }
    }
}

impl fmt::Display for TryLockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TryLockError::WouldBlock => "try_lock failed because the operation would block",
        }
        .fmt(f)
    }
}

impl core::error::Error for TryLockError {}

//! Traits related to synchronization primitives

use crate::{
    private::Sealed,
    time::{Duration, Instant},
};


/// Define basic operations for a mutex.
///
/// A type that implements this can be used with the [`Mutex`] type to have a mutex that functions
/// similarly as the Rust standard library mutex.
///
/// [`Mutex`]: crate::sync::poison::Mutex
pub trait RawMutex: Sealed {
    // HACK: using constants while we can't implement const functions on Traits
    /// A constant new to overcome the limitations of traits not having const fn yet on trits
    const NEW: Self;

    /// Locks the mutex or panics if the lock is already held by the current thread.
    fn lock(&self);

    /// Tries to acquire this mutex without blocking, returning `true` if the lock was successfull
    /// and `false` otherwise.
    fn try_lock(&self) -> bool;

    /// Unlocks this mutex.
    ///
    /// # Safety
    ///
    /// This method may only be called if the mutex is held in the current context, i.e. it must
    /// be paired with a successful call to [`lock`], [`try_lock`], [`try_lock_for`] or
    /// [`try_lock_until`].
    ///
    /// [`lock`]: RawMutex::lock
    /// [`try_lock`]: RawMutex::try_lock
    /// [`try_lock_for`]: RawMutexTimed::try_lock_for
    /// [`try_lock_until`]: RawMutexTimed::try_lock_until
    unsafe fn unlock(&self);

    fn is_locked(&self) -> bool {
        let was_acquired = self.try_lock();
        // we need to unlock now if it was acquired
        if was_acquired {
            // SAFETY: The lock was acquired with success above
            unsafe { self.unlock() };
        }
        !was_acquired
    }
}

/// Define timed operation on a mutex.
pub trait RawMutexTimed: RawMutex {
    /// Attempts to acquire this lock for a `timeout` duration.
    fn try_lock_for(&self, timeout: Duration) -> bool;

    /// Attempts to acquire this lock until a timeout is reached.
    #[inline]
    fn try_lock_until(&self, timeout: Instant) -> bool {
        let now = Instant::now();
        let duration = timeout.duration_since(now);

        // if zero, now is later than timeout, so we past the time to try
        if duration.is_zero() {
            false
        } else {
            self.try_lock_for(duration)
        }
    }
}

/// Define basic operations for a ReadWrite Lock.
///
/// A type that implements this can be used with the [`RwLock`] type to have a lock that functions
/// similarly as the Rust standard library rwlock.
///
/// [`RwLock`]: crate::sync::poison::RwLock
pub trait RawRwLock: Sealed {
    const NEW: Self;

    fn read(&self);
    fn try_read(&self) -> bool;

    fn write(&self);
    fn try_write(&self) -> bool;

    unsafe fn read_unlock(&self);
    unsafe fn write_unlock(&self);

    unsafe fn downgrade(&self);
}

pub trait RawRwLockTimed: RawRwLock {
    /// Attempts to acquire a read lock for a `timeout` duration.
    fn try_read_for(&self, timeout: Duration) -> bool;

    /// Attempts to acquire a read lock until a timeout is reached.
    fn try_read_until(&self, timeout: Instant) -> bool {
        let now = Instant::now();
        let duration = timeout.duration_since(now);

        // if zero, now is later than timeout, so we past the time to try
        if duration.is_zero() {
            false
        } else {
            self.try_read_for(duration)
        }
    }

    /// Attempts to acquire a write lock for a `timeout` duration.
    fn try_write_for(&self, timeout: Duration) -> bool;

    /// Attempts to acquire a write lock until a timeout is reached.
    fn try_write_until(&self, timeout: Instant) -> bool {
        let now = Instant::now();
        let duration = timeout.duration_since(now);

        // if zero, now is later than timeout, so we past the time to try
        if duration.is_zero() {
            false
        } else {
            self.try_write_for(duration)
        }
    }
}

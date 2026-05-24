use core::{
    cell::UnsafeCell,
    fmt,
    marker::PhantomData,
    mem::{self, ManuallyDrop},
    ops::{Deref, DerefMut},
    ptr::NonNull,
};


use crate::{
    sync::{
        poison::{self, LockResult, TryLockError, TryLockResult},
        RawRwLock, RawRwLockTimed,
    },
    sys::sync as sys,
    time::{Duration, Instant},
};

use super::PoisonError;

type DefaultRwLock = cfg_select! {
    feature = "kernel" => sys::RwLock,
    _ => sys::LwRwLock,
};


/// A reader-writer lock
///
/// This type of lock allows a number of readers or at most one writer at any
/// point in time. The write portion of this lock typically allows modification
/// of the underlying data (exclusive access) and the read portion of this lock
/// typically allows for read-only access (shared access).
///
/// In comparison, a [`Mutex`] does not distinguish between readers or writers
/// that acquire the lock, therefore blocking any threads waiting for the lock to
/// become available. An `RwLock` will allow any number of readers to acquire the
/// lock as long as a writer is not holding the lock.
///
/// The priority policy of the lock is dependent on the underlying operating
/// system's implementation, and this type does not guarantee that any
/// particular policy will be used. In particular, a writer which is waiting to
/// acquire the lock in `write` might or might not block concurrent calls to
/// `read`, e.g.:
///
/// <details><summary>Potential deadlock example</summary>
///
/// ```text
/// // Thread 1              |  // Thread 2
/// let _rg1 = lock.read();  |
///                          |  // will block
///                          |  let _wg = lock.write();
/// // may deadlock          |
/// let _rg2 = lock.read();  |
/// ```
///
/// </details>
///
/// The type parameter `T` represents the data that this lock protects. It is
/// required that `T` satisfies [`Send`] to be shared across threads and
/// [`Sync`] to allow concurrent access through readers. The RAII guards
/// returned from the locking methods implement [`Deref`] (and [`DerefMut`]
/// for the `write` methods) to allow access to the content of the lock.
///
/// # Poisoning
///
/// An `RwLock`, like [`Mutex`], will become poisoned on a panic. Note, however,
/// that an `RwLock` may only be poisoned if a panic occurs while it is locked
/// exclusively (write mode). If a panic occurs in any reader, then the lock
/// will not be poisoned.
///
/// # Examples
///
/// ```
/// use pspsdk::sync::poison::RwLock;
///
/// let lock = RwLock::new(5);
///
/// // many reader locks can be held at once
/// {
///     let r1 = lock.read();
///     let r2 = lock.read();
///     assert_eq!(*r1, 5);
///     assert_eq!(*r2, 5);
/// } // read locks are dropped at this point
///
/// // only one write lock may be held, however
/// {
///     let mut w = lock.write();
///     *w += 1;
///     assert_eq!(*w, 6);
/// } // write lock is dropped here
/// ```
///
/// [`Mutex`]: super::Mutex
pub struct RwLock<T: ?Sized, Raw = DefaultRwLock> {
    inner: Raw,
    poison: poison::Flag,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send, Raw> Send for RwLock<T, Raw> {}
unsafe impl<T: ?Sized + Send + Sync, Raw> Sync for RwLock<T, Raw> {}

impl<T: ?Sized, Raw> core::panic::UnwindSafe for RwLock<T, Raw> {}
impl<T: ?Sized, Raw> core::panic::RefUnwindSafe for RwLock<T, Raw> {}

/// RAII structure used to release the shared read access of a lock when
/// dropped.
///
/// This structure is created by the [`read`] and [`try_read`] methods on
/// [`RwLock`].
///
/// [`read`]: RwLock::read
/// [`try_read`]: RwLock::try_read
#[must_use = "if unused the RwLock will immediately unlock"]
#[clippy::has_significant_drop]
pub struct RwLockReadGuard<'a, T: ?Sized + 'a, Raw: RawRwLock = DefaultRwLock> {
    // NB: we use a pointer instead of `&'a T` to avoid `noalias` violations, because a
    // `RwLockReadGuard` argument doesn't hold immutability for its whole scope, only until it
    // drops. `NonNull` is also covariant over `T`, just like we would have with `&T`.
    // `NonNull` is preferable over `const* T` to allow for niche optimization.
    data: NonNull<T>,
    inner_lock: &'a Raw,
}

impl<T: ?Sized, L: RawRwLock> !Send for RwLockReadGuard<'_, T, L> {}
unsafe impl<T: ?Sized + Sync, L: RawRwLock> Sync for RwLockReadGuard<'_, T, L> {}

/// RAII structure used to release the exclusive write access of a lock when
/// dropped.
///
/// This structure is created by the [`write`] and [`try_write`] methods
/// on [`RwLock`].
///
/// [`write`]: RwLock::write
/// [`try_write`]: RwLock::try_write
#[must_use = "if unused the RwLock will immediately unlock"]
#[clippy::has_significant_drop]
pub struct RwLockWriteGuard<'a, T: ?Sized + 'a, L: RawRwLock = DefaultRwLock> {
    lock: &'a RwLock<T, L>,
    poison: poison::Guard,
}

impl<T: ?Sized, L: RawRwLock> !Send for RwLockWriteGuard<'_, T, L> {}
unsafe impl<T: ?Sized + Sync, L: RawRwLock> Sync for RwLockWriteGuard<'_, T, L> {}

/// RAII structure used to release the shared read access of a lock when
/// dropped, which can point to a subfield of the protected data.
///
/// This structure is created by the [`map`] and [`try_map`] methods
/// on [`RwLockReadGuard`].
///
/// [`map`]: RwLockReadGuard::map
/// [`try_map`]: RwLockReadGuard::try_map
#[must_use = "if unused the RwLock will immediately unlock"]
// #[must_not_suspend = "holding a MappedRwLockReadGuard across suspend points can cause deadlocks,
// \                       delays, and cause Futures to not implement `Send`"]
#[clippy::has_significant_drop]
pub struct MappedRwLockReadGuard<'a, T: ?Sized + 'a, L: RawRwLock = DefaultRwLock> {
    // NB: we use a pointer instead of `&'a T` to avoid `noalias` violations, because a
    // `MappedRwLockReadGuard` argument doesn't hold immutability for its whole scope, only until
    // it drops. `NonNull` is also covariant over `T`, just like we would have with `&T`.
    // `NonNull` is preferable over `const* T` to allow for niche optimization.
    data: NonNull<T>,
    inner_lock: &'a L,
}

impl<T: ?Sized, L: RawRwLock> !Send for MappedRwLockReadGuard<'_, T, L> {}
unsafe impl<T: ?Sized + Sync, L: RawRwLock> Sync for MappedRwLockReadGuard<'_, T, L> {}

/// RAII structure used to release the exclusive write access of a lock when
/// dropped, which can point to a subfield of the protected data.
///
/// This structure is created by the [`map`] and [`try_map`] methods
/// on [`RwLockWriteGuard`].
///
/// [`map`]: RwLockWriteGuard::map
/// [`try_map`]: RwLockWriteGuard::try_map
#[must_use = "if unused the RwLock will immediately unlock"]
#[clippy::has_significant_drop]
pub struct MappedRwLockWriteGuard<'a, T: ?Sized + 'a, L: RawRwLock = DefaultRwLock> {
    // NB: we use a pointer instead of `&'a mut T` to avoid `noalias` violations, because a
    // `MappedRwLockWriteGuard` argument doesn't hold uniqueness for its whole scope, only until it
    // drops. `NonNull` is covariant over `T`, so we add a `PhantomData<&'a mut T>` field
    // below for the correct variance over `T` (invariance).
    data: NonNull<T>,
    inner_lock: &'a L,
    poison_flag: &'a poison::Flag,
    poison: poison::Guard,
    _variance: PhantomData<&'a mut T>,
}

impl<T: ?Sized, L: RawRwLock> !Send for MappedRwLockWriteGuard<'_, T, L> {}
unsafe impl<T: ?Sized + Sync, L: RawRwLock> Sync for MappedRwLockWriteGuard<'_, T, L> {}

impl<T> RwLock<T> {
    /// Creates a new instance of an `RwLock<T>` which is unlocked.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::poison::RwLock;
    ///
    /// let lock = RwLock::new(5);
    /// ```
    #[inline]
    pub const fn new(t: T) -> RwLock<T> {
        RwLock {
            inner: sys::RwLock::new(),
            poison: poison::Flag::new(),
            data: UnsafeCell::new(t),
        }
    }
}

impl<T, L: RawRwLock> RwLock<T, L> {
    /// Creates a new instance of an `RwLock<T>` which is unlocked using the specified lock.
    #[inline]
    pub const fn new_with(value: T, raw_lock: L) -> Self {
        Self {
            inner: raw_lock,
            poison: poison::Flag::new(),
            data: UnsafeCell::new(value),
        }
    }

    /// Returns the contained value by cloning it.
    ///
    /// # Errors
    ///
    /// This function will return an error if the `RwLock` is poisoned. An
    /// `RwLock` is poisoned whenever a writer panics while holding an exclusive
    /// lock.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::poison::RwLock;
    ///
    /// let mut lock = RwLock::new(7);
    ///
    /// assert_eq!(lock.get_cloned().unwrap(), 7);
    /// ```
    pub fn get_cloned(&self) -> Result<T, PoisonError<()>>
    where
        T: Clone,
    {
        match self.read_checked() {
            Ok(guard) => Ok((*guard).clone()),
            Err(_) => Err(PoisonError::new(())),
        }
    }

    /// Sets the contained value.
    ///
    /// # Errors
    ///
    /// This function will return an error containing the provided `value` if
    /// the `RwLock` is poisoned. An `RwLock` is poisoned whenever a writer
    /// panics while holding an exclusive lock.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::poison::RwLock;
    ///
    /// let mut lock = RwLock::new(7);
    ///
    /// assert_eq!(lock.get_cloned().unwrap(), 7);
    /// lock.set(11).unwrap();
    /// assert_eq!(lock.get_cloned().unwrap(), 11);
    /// ```
    pub fn set(&self, value: T) -> Result<(), PoisonError<T>> {
        if mem::needs_drop::<T>() {
            // If the contained value has non-trivial destructor, we
            // call that destructor after the lock being released.
            self.replace(value).map(drop)
        } else {
            match self.write_checked() {
                Ok(mut guard) => {
                    *guard = value;

                    Ok(())
                },
                Err(_) => Err(PoisonError::new(value)),
            }
        }
    }

    /// Replaces the contained value with `value`, and returns the old contained value.
    ///
    /// # Errors
    ///
    /// This function will return an error containing the provided `value` if
    /// the `RwLock` is poisoned. An `RwLock` is poisoned whenever a writer
    /// panics while holding an exclusive lock.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::poison::RwLock;
    ///
    /// let mut lock = RwLock::new(7);
    ///
    /// assert_eq!(lock.replace(11).unwrap(), 7);
    /// assert_eq!(lock.get_cloned().unwrap(), 11);
    /// ```
    pub fn replace(&self, value: T) -> LockResult<T> {
        match self.write_checked() {
            Ok(mut guard) => Ok(mem::replace(&mut *guard, value)),
            Err(_) => Err(PoisonError::new(value)),
        }
    }
}

impl<T: ?Sized, L: RawRwLock> RwLock<T, L> {
    /// Locks this `RwLock` with shared read access, blocking the current thread
    /// until it can be acquired.
    ///
    /// The calling thread will be blocked until there are no more writers which
    /// hold the lock. There may be other readers currently inside the lock when
    /// this method returns. This method does not provide any guarantees with
    /// respect to the ordering of whether contentious readers or writers will
    /// acquire the lock first.
    ///
    /// Returns an RAII guard which will release this thread's shared access
    /// once it is dropped.
    ///
    /// # Panics
    ///
    /// This function might panic when called if the lock is already held by the current thread.
    ///
    /// This function will panic if the `RwLock` is poisoned. An
    /// `RwLock` is poisoned whenever a writer panics while holding an exclusive
    /// lock. The failure will occur immediately after the lock has been
    /// acquired.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{sync::Arc, thread};
    ///
    /// use pspsdk::sync::poison::RwLock;
    ///
    /// let lock = Arc::new(RwLock::new(1));
    /// let c_lock = Arc::clone(&lock);
    ///
    /// let n = lock.read();
    /// assert_eq!(*n, 1);
    ///
    /// thread::spawn(move || {
    ///     let r = c_lock.read();
    ///     assert_eq!(*r, 1);
    /// })
    /// .join()
    /// .unwrap();
    /// ```
    #[inline]
    pub fn read(&self) -> RwLockReadGuard<'_, T, L> {
        unsafe {
            self.inner.read();
            match RwLockReadGuard::new(self) {
                Ok(guard) => guard,
                Err(err) => panic!("{err}"),
            }
        }
    }

    /// Attempts to acquire this `RwLock` with shared read access.
    ///
    /// If the access could not be granted at this time, then `Err` is returned.
    /// Otherwise, an RAII guard is returned which will release the shared access
    /// when it is dropped.
    ///
    /// This function does not block.
    ///
    /// This function does not provide any guarantees with respect to the ordering
    /// of whether contentious readers or writers will acquire the lock first.
    ///
    /// # Errors
    ///
    /// This function will return the [`Poisoned`] error if the `RwLock` is
    /// poisoned. An `RwLock` is poisoned whenever a writer panics while holding
    /// an exclusive lock. `Poisoned` will only be returned if the lock would
    /// have otherwise been acquired. An acquired lock guard will be contained
    /// in the returned error.
    ///
    /// This function will return the [`WouldBlock`] error if the `RwLock` could
    /// not be acquired because it was already locked exclusively.
    ///
    /// [`Poisoned`]: TryLockError::Poisoned
    /// [`WouldBlock`]: TryLockError::WouldBlock
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::poison::RwLock;
    ///
    /// let lock = RwLock::new(1);
    ///
    /// match lock.try_read() {
    ///     Ok(n) => assert_eq!(*n, 1),
    ///     Err(_) => unreachable!(),
    /// };
    /// ```
    #[inline]
    pub fn try_read(&self) -> TryLockResult<RwLockReadGuard<'_, T, L>> {
        unsafe {
            if self.inner.try_read() {
                Ok(RwLockReadGuard::new(self)?)
            } else {
                Err(TryLockError::WouldBlock)
            }
        }
    }

    /// Locks this `RwLock` with exclusive write access, blocking the current
    /// thread until it can be acquired.
    ///
    /// This function will not return while other writers or other readers
    /// currently have access to the lock.
    ///
    /// Returns an RAII guard which will drop the write access of this `RwLock`
    /// when dropped.
    ///
    /// # Panics
    ///
    /// This function might panic when called if the lock is already held by the current thread.
    ///
    /// This function will panic if the `RwLock` is poisoned. An `RwLock` is poisoned whenever a
    /// writer panics while holding an exclusive lock. An error will be returned when the lock is
    /// acquired.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::poison::RwLock;
    ///
    /// let lock = RwLock::new(1);
    ///
    /// let mut n = lock.write();
    /// *n = 2;
    ///
    /// assert!(lock.try_read().is_err());
    /// ```
    #[inline]
    pub fn write(&self) -> RwLockWriteGuard<'_, T, L> {
        unsafe {
            self.inner.write();
            match RwLockWriteGuard::new(self) {
                Ok(guard) => guard,
                Err(err) => panic!("{err}"),
            }
        }
    }

    /// Attempts to lock this `RwLock` with exclusive write access.
    ///
    /// If the lock could not be acquired at this time, then `Err` is returned.
    /// Otherwise, an RAII guard is returned which will release the lock when
    /// it is dropped.
    ///
    /// This function does not block.
    ///
    /// This function does not provide any guarantees with respect to the ordering
    /// of whether contentious readers or writers will acquire the lock first.
    ///
    /// # Errors
    ///
    /// This function will return the [`Poisoned`] error if the `RwLock` is
    /// poisoned. An `RwLock` is poisoned whenever a writer panics while holding
    /// an exclusive lock. `Poisoned` will only be returned if the lock would
    /// have otherwise been acquired. An acquired lock guard will be contained
    /// in the returned error.
    ///
    /// This function will return the [`WouldBlock`] error if the `RwLock` could
    /// not be acquired because it was already locked exclusively.
    ///
    /// [`Poisoned`]: TryLockError::Poisoned
    /// [`WouldBlock`]: TryLockError::WouldBlock
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::poison::RwLock;
    ///
    /// let lock = RwLock::new(1);
    ///
    /// let n = lock.read();
    /// assert_eq!(*n, 1);
    ///
    /// assert!(lock.try_write().is_err());
    /// ```
    #[inline]
    pub fn try_write(&self) -> TryLockResult<RwLockWriteGuard<'_, T, L>> {
        unsafe {
            if self.inner.try_write() {
                Ok(RwLockWriteGuard::new(self)?)
            } else {
                Err(TryLockError::WouldBlock)
            }
        }
    }

    /// Locks this `RwLock` with shared read access ignoring poison state, blocking the current
    /// thread until it can be acquired.
    ///
    /// The calling thread will be blocked until there are no more writers which
    /// hold the lock. There may be other readers currently inside the lock when
    /// this method returns. This method does not provide any guarantees with
    /// respect to the ordering of whether contentious readers or writers will
    /// acquire the lock first.
    ///
    /// Returns an RAII guard which will release this thread's shared access
    /// once it is dropped.
    ///
    /// # Panics
    ///
    /// This function might panic when called if the lock is already held by the current thread.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{sync::Arc, thread};
    ///
    /// use pspsdk::sync::poison::RwLock;
    ///
    /// let lock = Arc::new(RwLock::new(1));
    /// let c_lock = Arc::clone(&lock);
    ///
    /// let n = lock.read();
    /// assert_eq!(*n, 1);
    ///
    /// thread::spawn(move || {
    ///     let r = c_lock.read_unchecked();
    ///     assert_eq!(*r, 1);
    /// })
    /// .join()
    /// .unwrap();
    /// ```
    #[inline]
    pub fn read_unchecked(&self) -> RwLockReadGuard<'_, T, L> {
        unsafe {
            self.inner.read();
            match RwLockReadGuard::new(self) {
                Ok(guard) => guard,
                Err(err) => err.into_inner(),
            }
        }
    }

    /// Locks this `RwLock` with exclusive write access ignoring poison state, blocking the current
    /// thread until it can be acquired.
    ///
    /// This function will not return while other writers or other readers
    /// currently have access to the lock.
    ///
    /// Returns an RAII guard which will drop the write access of this `RwLock`
    /// when dropped.
    ///
    /// # Panics
    ///
    /// This function might panic when called if the lock is already held by the current thread.
    ///
    /// This function will panic if the `RwLock` is poisoned. An `RwLock` is poisoned whenever a
    /// writer panics while holding an exclusive lock. An error will be returned when the lock is
    /// acquired.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::poison::RwLock;
    ///
    /// let lock = RwLock::new(1);
    ///
    /// let mut n = lock.write();
    /// *n = 2;
    ///
    /// assert!(lock.try_read().is_err());
    /// ```
    #[inline]
    pub fn write_unchecked(&self) -> RwLockWriteGuard<'_, T, L> {
        unsafe {
            self.inner.write();
            match RwLockWriteGuard::new(self) {
                Ok(guard) => guard,
                Err(err) => err.into_inner(),
            }
        }
    }

    /// Consumes this `RwLock`, returning the underlying data.
    ///
    /// # Panics
    ///
    /// This function will panic if the `RwLock` is poisoned. An
    /// `RwLock` is poisoned whenever a writer panics while holding an exclusive
    /// lock.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::poison::RwLock;
    ///
    /// let lock = RwLock::new(String::new());
    /// {
    ///     let mut s = lock.write();
    ///     *s = "modified".to_owned();
    /// }
    /// assert_eq!(lock.into_inner(), "modified");
    /// ```
    pub fn into_inner(self) -> T
    where
        T: Sized,
    {
        let data = self.data.into_inner();
        match poison::map_result(self.poison.borrow(), |()| data) {
            Ok(data) => data,
            Err(err) => panic!("{err}"),
        }
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the `RwLock` mutably, no actual locking needs to
    /// take place -- the mutable borrow statically guarantees no locks exist.
    ///
    /// # Panics
    ///
    /// This function will panic if the `RwLock` is poisoned. An
    /// `RwLock` is poisoned whenever a writer panics while holding an exclusive
    /// lock.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::poison::RwLock;
    ///
    /// let mut lock = RwLock::new(0);
    /// *lock.get_mut() = 10;
    /// assert_eq!(*lock.read(), 10);
    /// ```
    pub fn get_mut(&mut self) -> &mut T {
        let data = self.data.get_mut();
        match poison::map_result(self.poison.borrow(), |()| data) {
            Ok(data) => data,
            Err(err) => panic!("{err}"),
        }
    }

    /// Consumes this `RwLock`, returning the underlying data without checks if the lock is
    /// poisoned.
    pub fn into_inner_unchecked(self) -> T
    where
        T: Sized,
    {
        self.data.into_inner()
    }

    /// Returns a mutable reference to the underlying data without checks if the lock is
    /// poisoned.
    ///
    /// Since this call borrows the `RwLock` mutably, no actual locking needs to
    /// take place -- the mutable borrow statically guarantees no locks exist.
    pub fn get_mut_unchecked(&mut self) -> &mut T {
        self.data.get_mut()
    }
}

impl<T: ?Sized, L: RawRwLock> RwLock<T, L> {
    /// Locks this `RwLock` with shared read access, blocking the current thread
    /// until it can be acquired.
    ///
    /// The calling thread will be blocked until there are no more writers which
    /// hold the lock. There may be other readers currently inside the lock when
    /// this method returns. This method does not provide any guarantees with
    /// respect to the ordering of whether contentious readers or writers will
    /// acquire the lock first.
    ///
    /// Returns an RAII guard which will release this thread's shared access
    /// once it is dropped.
    ///
    /// # Errors
    ///
    /// This function will return an error if the `RwLock` is poisoned. An
    /// `RwLock` is poisoned whenever a writer panics while holding an exclusive
    /// lock. The failure will occur immediately after the lock has been
    /// acquired. The acquired lock guard will be contained in the returned
    /// error.
    ///
    /// # Panics
    ///
    /// This function might panic when called if the lock is already held by the current thread.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{sync::Arc, thread};
    ///
    /// use pspsdk::sync::poison::RwLock;
    ///
    /// let lock = Arc::new(RwLock::new(1));
    /// let c_lock = Arc::clone(&lock);
    ///
    /// let n = lock.read();
    /// assert_eq!(*n, 1);
    ///
    /// thread::spawn(move || {
    ///     let r = c_lock.read_checked();
    ///     assert!(r.is_ok());
    /// })
    /// .join()
    /// .unwrap();
    /// ```
    #[inline]
    pub fn read_checked(&self) -> LockResult<RwLockReadGuard<'_, T, L>> {
        unsafe {
            self.inner.read();
            RwLockReadGuard::new(self)
        }
    }

    /// Locks this `RwLock` with exclusive write access, blocking the current
    /// thread until it can be acquired.
    ///
    /// This function will not return while other writers or other readers
    /// currently have access to the lock.
    ///
    /// Returns an RAII guard which will drop the write access of this `RwLock`
    /// when dropped.
    ///
    /// # Errors
    ///
    /// This function will return an error if the `RwLock` is poisoned. An
    /// `RwLock` is poisoned whenever a writer panics while holding an exclusive
    /// lock. An error will be returned when the lock is acquired. The acquired
    /// lock guard will be contained in the returned error.
    ///
    /// # Panics
    ///
    /// This function might panic when called if the lock is already held by the current thread.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::poison::RwLock;
    ///
    /// let lock = RwLock::new(1);
    ///
    /// let mut n = lock.write();
    /// *n = 2;
    ///
    /// assert!(lock.try_read().is_err());
    /// ```
    #[inline]
    pub fn write_checked(&self) -> LockResult<RwLockWriteGuard<'_, T, L>> {
        unsafe {
            self.inner.write();
            RwLockWriteGuard::new(self)
        }
    }

    /// Determines whether the lock is poisoned.
    ///
    /// If another thread is active, the lock can still become poisoned at any
    /// time. You should not trust a `false` value for program correctness
    /// without additional synchronization.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{
    ///     sync::{Arc, RwLock},
    ///     thread,
    /// };
    ///
    /// let lock = Arc::new(RwLock::new(0));
    /// let c_lock = Arc::clone(&lock);
    ///
    /// let _ = thread::spawn(move || {
    ///     let _lock = c_lock.write().unwrap();
    ///     panic!(); // the lock gets poisoned
    /// })
    /// .join();
    /// assert_eq!(lock.is_poisoned(), true);
    /// ```
    #[inline]
    pub fn is_poisoned(&self) -> bool {
        self.poison.get()
    }

    /// Clear the poisoned state from a lock.
    ///
    /// If the lock is poisoned, it will remain poisoned until this function is called. This allows
    /// recovering from a poisoned state and marking that it has recovered. For example, if the
    /// value is overwritten by a known-good value, then the lock can be marked as un-poisoned. Or
    /// possibly, the value could be inspected to determine if it is in a consistent state, and if
    /// so the poison is removed.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{
    ///     sync::{Arc, RwLock},
    ///     thread,
    /// };
    ///
    /// let lock = Arc::new(RwLock::new(0));
    /// let c_lock = Arc::clone(&lock);
    ///
    /// let _ = thread::spawn(move || {
    ///     let _lock = c_lock.write().unwrap();
    ///     panic!(); // the lock gets poisoned
    /// })
    /// .join();
    ///
    /// assert_eq!(lock.is_poisoned(), true);
    /// let guard = lock.write().unwrap_or_else(|mut e| {
    ///     **e.get_mut() = 1;
    ///     lock.clear_poison();
    ///     e.into_inner()
    /// });
    /// assert_eq!(lock.is_poisoned(), false);
    /// assert_eq!(*guard, 1);
    /// ```
    #[inline]
    pub fn clear_poison(&self) {
        self.poison.clear();
    }

    /// Consumes this `RwLock`, returning the underlying data.
    ///
    /// # Errors
    ///
    /// This function will return an error containing the underlying data if
    /// the `RwLock` is poisoned. An `RwLock` is poisoned whenever a writer
    /// panics while holding an exclusive lock. An error will only be returned
    /// if the lock would have otherwise been acquired.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::poison::RwLock;
    ///
    /// let lock = RwLock::new(String::new());
    /// {
    ///     let mut s = lock.write();
    ///     *s = "modified".to_owned();
    /// }
    /// assert_eq!(lock.into_inner_checked().unwrap(), "modified");
    /// ```
    pub fn into_inner_checked(self) -> LockResult<T>
    where
        T: Sized,
    {
        let data = self.data.into_inner();
        poison::map_result(self.poison.borrow(), |()| data)
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the `RwLock` mutably, no actual locking needs to
    /// take place -- the mutable borrow statically guarantees no locks exist.
    ///
    /// # Errors
    ///
    /// This function will return an error containing a mutable reference to
    /// the underlying data if the `RwLock` is poisoned. An `RwLock` is
    /// poisoned whenever a writer panics while holding an exclusive lock.
    /// An error will only be returned if the lock would have otherwise been
    /// acquired.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::poison::RwLock;
    ///
    /// let mut lock = RwLock::new(0);
    /// *lock.get_mut_checked().unwrap() = 10;
    /// assert_eq!(*lock.read(), 10);
    /// ```
    pub fn get_mut_checked(&mut self) -> LockResult<&mut T> {
        let data = self.data.get_mut();
        poison::map_result(self.poison.borrow(), |()| data)
    }

    /// Returns a raw pointer to the underlying data.
    ///
    /// The returned pointer is always non-null and properly aligned, but it is
    /// the user's responsibility to ensure that any reads and writes through it
    /// are properly synchronized to avoid data races, and that it is not read
    /// or written through after the mutex is dropped.
    pub const fn data_ptr(&self) -> *mut T {
        self.data.get()
    }
}

impl<T: ?Sized, L: RawRwLockTimed> RwLock<T, L> {
    /// Attempts to acquire this `RwLock` with shared read access until a timeout
    /// is reached.
    ///
    /// If the access could not be granted before the timeout expires, then
    /// `None` is returned. Otherwise, an RAII guard is returned which will
    /// release the shared access when it is dropped.
    #[inline]
    #[track_caller]
    pub fn try_read_for(&self, timeout: Duration) -> Option<RwLockReadGuard<'_, T, L>> {
        if self.inner.try_lock_for(timeout) {
            // SAFETY: The lock is held, as required.
            unsafe { RwLockReadGuard::new(self).ok() }
        } else {
            None
        }
    }

    /// Attempts to acquire this `RwLock` with shared read access until a timeout
    /// is reached.
    ///
    /// If the access could not be granted before the timeout expires, then
    /// `None` is returned. Otherwise, an RAII guard is returned which will
    /// release the shared access when it is dropped.
    #[inline]
    #[track_caller]
    pub fn try_read_until(&self, timeout: Instant) -> Option<RwLockReadGuard<'_, T, L>> {
        if self.inner.try_lock_until(timeout) {
            // SAFETY: The lock is held, as required.
            unsafe { RwLockReadGuard::new(self).ok() }
        } else {
            None
        }
    }

    /// Attempts to acquire this `RwLock` with exclusive write access until a
    /// timeout is reached.
    ///
    /// If the access could not be granted before the timeout expires, then
    /// `None` is returned. Otherwise, an RAII guard is returned which will
    /// release the exclusive access when it is dropped.
    #[inline]
    #[track_caller]
    pub fn try_write_for(&self, timeout: Duration) -> Option<RwLockWriteGuard<'_, T, L>> {
        if self.inner.try_write_for(timeout) {
            // SAFETY: The lock is held, as required.
            unsafe { RwLockWriteGuard::new(self).ok() }
        } else {
            None
        }
    }

    /// Attempts to acquire this `RwLock` with exclusive write access until a
    /// timeout is reached.
    ///
    /// If the access could not be granted before the timeout expires, then
    /// `None` is returned. Otherwise, an RAII guard is returned which will
    /// release the exclusive access when it is dropped.
    #[inline]
    #[track_caller]
    pub fn try_write_until(&self, timeout: Instant) -> Option<RwLockWriteGuard<'_, T, L>> {
        if self.inner.try_write_until(timeout) {
            // SAFETY: The lock is held, as required.
            unsafe { RwLockWriteGuard::new(self).ok() }
        } else {
            None
        }
    }
}

impl<T: ?Sized + fmt::Debug, L: RawRwLock> fmt::Debug for RwLock<T, L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("RwLock");
        match self.try_read() {
            Ok(guard) => {
                d.field("data", &&*guard);
            },
            Err(TryLockError::Poisoned(err)) => {
                d.field("data", &&**err.get_ref());
            },
            Err(TryLockError::WouldBlock) => {
                d.field("data", &format_args!("<locked>"));
            },
        }
        d.field("poisoned", &self.poison.get());
        d.finish_non_exhaustive()
    }
}

impl<T: Default> Default for RwLock<T> {
    /// Creates a new `RwLock<T>`, with the `Default` value for T.
    fn default() -> RwLock<T> {
        RwLock::new(Default::default())
    }
}

impl<T> From<T> for RwLock<T> {
    /// Creates a new instance of an `RwLock<T>` which is unlocked.
    /// This is equivalent to [`RwLock::new`].
    fn from(t: T) -> Self {
        RwLock::new(t)
    }
}

impl<'rwlock, T: ?Sized, L: RawRwLock> RwLockReadGuard<'rwlock, T, L> {
    /// Create a new instance of `RwLockReadGuard<T>` from a `RwLock<T>`.
    ///
    /// # Safety
    ///
    /// This function is safe if and only if the same thread has successfully and safely called
    /// `lock.inner.read()`, `lock.inner.try_read()`, or `lock.inner.downgrade()` before
    /// instantiating this object.
    unsafe fn new(lock: &'rwlock RwLock<T, L>) -> LockResult<RwLockReadGuard<'rwlock, T, L>> {
        poison::map_result(lock.poison.borrow(), |()| RwLockReadGuard {
            data: unsafe { NonNull::new_unchecked(lock.data.get()) },
            inner_lock: &lock.inner,
        })
    }
}

impl<'rwlock, T: ?Sized, L: RawRwLock> RwLockWriteGuard<'rwlock, T, L> {
    /// Create a new instance of `RwLockWriteGuard<T>` from a `RwLock<T>`.
    // SAFETY: if and only if `lock.inner.write()` (or `lock.inner.try_write()`) has been
    // successfully called from the same thread before instantiating this object.
    unsafe fn new(lock: &'rwlock RwLock<T, L>) -> LockResult<RwLockWriteGuard<'rwlock, T, L>> {
        poison::map_result(lock.poison.guard(), |guard| RwLockWriteGuard {
            lock,
            poison: guard,
        })
    }
}

impl<T, L> fmt::Debug for RwLockReadGuard<'_, T, L>
where
    T: ?Sized + fmt::Debug,
    L: RawRwLock,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T, L> fmt::Display for RwLockReadGuard<'_, T, L>
where
    T: ?Sized + fmt::Display,
    L: RawRwLock,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T, L> fmt::Debug for RwLockWriteGuard<'_, T, L>
where
    T: ?Sized + fmt::Debug,
    L: RawRwLock,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T, L> fmt::Display for RwLockWriteGuard<'_, T, L>
where
    T: ?Sized + fmt::Display,
    L: RawRwLock,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T, L> fmt::Debug for MappedRwLockReadGuard<'_, T, L>
where
    T: ?Sized + fmt::Debug,
    L: RawRwLock,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T, L> fmt::Display for MappedRwLockReadGuard<'_, T, L>
where
    T: ?Sized + fmt::Display,
    L: RawRwLock,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T, L> fmt::Debug for MappedRwLockWriteGuard<'_, T, L>
where
    T: ?Sized + fmt::Debug,
    L: RawRwLock,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T, L> fmt::Display for MappedRwLockWriteGuard<'_, T, L>
where
    T: ?Sized + fmt::Display,
    L: RawRwLock,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: ?Sized, L: RawRwLock> Deref for RwLockReadGuard<'_, T, L> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when created.
        unsafe { self.data.as_ref() }
    }
}

impl<T: ?Sized, L: RawRwLock> Deref for RwLockWriteGuard<'_, T, L> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when created.
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized, L: RawRwLock> DerefMut for RwLockWriteGuard<'_, T, L> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when created.
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T: ?Sized, L: RawRwLock> Deref for MappedRwLockReadGuard<'_, T, L> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        unsafe { self.data.as_ref() }
    }
}

impl<T: ?Sized, L: RawRwLock> Deref for MappedRwLockWriteGuard<'_, T, L> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        unsafe { self.data.as_ref() }
    }
}

impl<T: ?Sized, L: RawRwLock> DerefMut for MappedRwLockWriteGuard<'_, T, L> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        unsafe { self.data.as_mut() }
    }
}

impl<T: ?Sized, L: RawRwLock> Drop for RwLockReadGuard<'_, T, L> {
    fn drop(&mut self) {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when created.
        unsafe {
            self.inner_lock.read_unlock();
        }
    }
}

impl<T: ?Sized, L: RawRwLock> Drop for RwLockWriteGuard<'_, T, L> {
    fn drop(&mut self) {
        self.lock.poison.done(&self.poison);
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when created.
        unsafe {
            self.lock.inner.write_unlock();
        }
    }
}

impl<T: ?Sized, L: RawRwLock> Drop for MappedRwLockReadGuard<'_, T, L> {
    fn drop(&mut self) {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        unsafe {
            self.inner_lock.read_unlock();
        }
    }
}

impl<T: ?Sized, L: RawRwLock> Drop for MappedRwLockWriteGuard<'_, T, L> {
    fn drop(&mut self) {
        self.poison_flag.done(&self.poison);
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        unsafe {
            self.inner_lock.write_unlock();
        }
    }
}

impl<'a, T: ?Sized, L: RawRwLock> RwLockReadGuard<'a, T, L> {
    /// Makes a [`MappedRwLockReadGuard`] for a component of the borrowed data, e.g.
    /// an enum variant.
    ///
    /// The `RwLock` is already locked for reading, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `RwLockReadGuard::map(...)`. A method would interfere with methods of
    /// the same name on the contents of the `RwLockReadGuard` used through
    /// `Deref`.
    ///
    /// # Panics
    ///
    /// If the closure panics, the guard will be dropped (unlocked) and the RwLock will not be
    /// poisoned.
    pub fn map<U, F>(orig: Self, f: F) -> MappedRwLockReadGuard<'a, U, L>
    where
        F: FnOnce(&T) -> &U,
        U: ?Sized,
    {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the
        // reference passed to it. If the closure panics, the guard will be dropped.
        let data = NonNull::from(f(unsafe { orig.data.as_ref() }));
        let orig = ManuallyDrop::new(orig);
        MappedRwLockReadGuard {
            data,
            inner_lock: orig.inner_lock,
        }
    }

    /// Makes a [`MappedRwLockReadGuard`] for a component of the borrowed data. The
    /// original guard is returned as an `Err(...)` if the closure returns
    /// `None`.
    ///
    /// The `RwLock` is already locked for reading, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `RwLockReadGuard::try_map(...)`. A method would interfere with methods
    /// of the same name on the contents of the `RwLockReadGuard` used through
    /// `Deref`.
    ///
    /// # Panics
    ///
    /// If the closure panics, the guard will be dropped (unlocked) and the RwLock will not be
    /// poisoned.
    #[doc(alias = "filter_map")]
    pub fn try_map<U, F>(orig: Self, f: F) -> Result<MappedRwLockReadGuard<'a, U, L>, Self>
    where
        F: FnOnce(&T) -> Option<&U>,
        U: ?Sized,
    {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the
        // reference passed to it. If the closure panics, the guard will be dropped.
        match f(unsafe { orig.data.as_ref() }) {
            Some(data) => {
                let data = NonNull::from(data);
                let orig = ManuallyDrop::new(orig);
                Ok(MappedRwLockReadGuard {
                    data,
                    inner_lock: orig.inner_lock,
                })
            },
            None => Err(orig),
        }
    }
}

impl<'a, T: ?Sized, L: RawRwLock> MappedRwLockReadGuard<'a, T, L> {
    /// Makes a [`MappedRwLockReadGuard`] for a component of the borrowed data,
    /// e.g. an enum variant.
    ///
    /// The `RwLock` is already locked for reading, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `MappedRwLockReadGuard::map(...)`. A method would interfere with
    /// methods of the same name on the contents of the `MappedRwLockReadGuard`
    /// used through `Deref`.
    ///
    /// # Panics
    ///
    /// If the closure panics, the guard will be dropped (unlocked) and the RwLock will not be
    /// poisoned.
    pub fn map<U, F>(orig: Self, f: F) -> MappedRwLockReadGuard<'a, U, L>
    where
        F: FnOnce(&T) -> &U,
        U: ?Sized,
    {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the
        // reference passed to it. If the closure panics, the guard will be dropped.
        let data = NonNull::from(f(unsafe { orig.data.as_ref() }));
        let orig = ManuallyDrop::new(orig);
        MappedRwLockReadGuard {
            data,
            inner_lock: orig.inner_lock,
        }
    }

    /// Makes a [`MappedRwLockReadGuard`] for a component of the borrowed data.
    /// The original guard is returned as an `Err(...)` if the closure returns
    /// `None`.
    ///
    /// The `RwLock` is already locked for reading, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `MappedRwLockReadGuard::try_map(...)`. A method would interfere with
    /// methods of the same name on the contents of the `MappedRwLockReadGuard`
    /// used through `Deref`.
    ///
    /// # Panics
    ///
    /// If the closure panics, the guard will be dropped (unlocked) and the RwLock will not be
    /// poisoned.
    #[doc(alias = "filter_map")]
    pub fn try_map<U, F>(orig: Self, f: F) -> Result<MappedRwLockReadGuard<'a, U, L>, Self>
    where
        F: FnOnce(&T) -> Option<&U>,
        U: ?Sized,
    {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the
        // reference passed to it. If the closure panics, the guard will be dropped.
        match f(unsafe { orig.data.as_ref() }) {
            Some(data) => {
                let data = NonNull::from(data);
                let orig = ManuallyDrop::new(orig);
                Ok(MappedRwLockReadGuard {
                    data,
                    inner_lock: orig.inner_lock,
                })
            },
            None => Err(orig),
        }
    }
}

impl<'a, T: ?Sized, L: RawRwLock> RwLockWriteGuard<'a, T, L> {
    /// Makes a [`MappedRwLockWriteGuard`] for a component of the borrowed data, e.g.
    /// an enum variant.
    ///
    /// The `RwLock` is already locked for writing, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `RwLockWriteGuard::map(...)`. A method would interfere with methods of
    /// the same name on the contents of the `RwLockWriteGuard` used through
    /// `Deref`.
    ///
    /// # Panics
    ///
    /// If the closure panics, the guard will be dropped (unlocked) and the RwLock will be poisoned.
    pub fn map<U, F>(orig: Self, f: F) -> MappedRwLockWriteGuard<'a, U, L>
    where
        F: FnOnce(&mut T) -> &mut U,
        U: ?Sized,
    {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the
        // reference passed to it. If the closure panics, the guard will be dropped.
        let data = NonNull::from(f(unsafe { &mut *orig.lock.data.get() }));
        let orig = ManuallyDrop::new(orig);
        MappedRwLockWriteGuard {
            data,
            inner_lock: &orig.lock.inner,
            poison_flag: &orig.lock.poison,
            poison: orig.poison.clone(),
            _variance: PhantomData,
        }
    }

    /// Makes a [`MappedRwLockWriteGuard`] for a component of the borrowed data. The
    /// original guard is returned as an `Err(...)` if the closure returns
    /// `None`.
    ///
    /// The `RwLock` is already locked for writing, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `RwLockWriteGuard::try_map(...)`. A method would interfere with methods
    /// of the same name on the contents of the `RwLockWriteGuard` used through
    /// `Deref`.
    ///
    /// # Panics
    ///
    /// If the closure panics, the guard will be dropped (unlocked) and the RwLock will be poisoned.
    #[doc(alias = "filter_map")]
    pub fn try_map<U, F>(orig: Self, f: F) -> Result<MappedRwLockWriteGuard<'a, U, L>, Self>
    where
        F: FnOnce(&mut T) -> Option<&mut U>,
        U: ?Sized,
    {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the
        // reference passed to it. If the closure panics, the guard will be dropped.
        match f(unsafe { &mut *orig.lock.data.get() }) {
            Some(data) => {
                let data = NonNull::from(data);
                let orig = ManuallyDrop::new(orig);
                Ok(MappedRwLockWriteGuard {
                    data,
                    inner_lock: &orig.lock.inner,
                    poison_flag: &orig.lock.poison,
                    poison: orig.poison.clone(),
                    _variance: PhantomData,
                })
            },
            None => Err(orig),
        }
    }

    /// Downgrades a write-locked `RwLockWriteGuard` into a read-locked [`RwLockReadGuard`].
    ///
    /// This method will atomically change the state of the [`RwLock`] from exclusive mode into
    /// shared mode. This means that it is impossible for a writing thread to get in between a
    /// thread calling `downgrade` and the same thread reading whatever it wrote while it had the
    /// [`RwLock`] in write mode.
    ///
    /// Note that since we have the `RwLockWriteGuard`, we know that the [`RwLock`] is already
    /// locked for writing, so this method cannot fail.
    ///
    /// # Example
    ///
    /// ```
    /// #![feature(rwlock_downgrade)]
    /// use std::sync::{Arc, RwLock, RwLockWriteGuard};
    ///
    /// // The inner value starts as 0.
    /// let rw = Arc::new(RwLock::new(0));
    ///
    /// // Put the lock in write mode.
    /// let mut main_write_guard = rw.write().unwrap();
    ///
    /// let evil = rw.clone();
    /// let handle = std::thread::spawn(move || {
    ///     // This will not return until the main thread drops the `main_read_guard`.
    ///     let mut evil_guard = evil.write().unwrap();
    ///
    ///     assert_eq!(*evil_guard, 1);
    ///     *evil_guard = 2;
    /// });
    ///
    /// // After spawning the writer thread, set the inner value to 1.
    /// *main_write_guard = 1;
    ///
    /// // Atomically downgrade the write guard into a read guard.
    /// let main_read_guard = RwLockWriteGuard::downgrade(main_write_guard);
    ///
    /// // Since `downgrade` is atomic, the writer thread cannot have set the inner value to 2.
    /// assert_eq!(*main_read_guard, 1, "`downgrade` was not atomic");
    ///
    /// // Clean up everything now
    /// drop(main_read_guard);
    /// handle.join().unwrap();
    ///
    /// let final_check = rw.read().unwrap();
    /// assert_eq!(*final_check, 2);
    /// ```
    pub fn downgrade(s: Self) -> RwLockReadGuard<'a, T, L> {
        let lock = s.lock;

        // We don't want to call the destructor since that calls `write_unlock`.
        core::mem::forget(s);

        // SAFETY: We take ownership of a write guard, so we must already have the `RwLock` in write
        // mode, satisfying the `downgrade` contract.
        unsafe { lock.inner.downgrade() };

        // SAFETY: We have just successfully called `downgrade`, so we fulfill the safety contract.
        unsafe { RwLockReadGuard::new(lock).unwrap_or_else(poison::PoisonError::into_inner) }
    }
}

impl<'a, T: ?Sized, L: RawRwLock> MappedRwLockWriteGuard<'a, T, L> {
    /// Makes a [`MappedRwLockWriteGuard`] for a component of the borrowed data,
    /// e.g. an enum variant.
    ///
    /// The `RwLock` is already locked for writing, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `MappedRwLockWriteGuard::map(...)`. A method would interfere with
    /// methods of the same name on the contents of the `MappedRwLockWriteGuard`
    /// used through `Deref`.
    ///
    /// # Panics
    ///
    /// If the closure panics, the guard will be dropped (unlocked) and the RwLock will be poisoned.
    pub fn map<U, F>(mut orig: Self, f: F) -> MappedRwLockWriteGuard<'a, U, L>
    where
        F: FnOnce(&mut T) -> &mut U,
        U: ?Sized,
    {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the
        // reference passed to it. If the closure panics, the guard will be dropped.
        let data = NonNull::from(f(unsafe { orig.data.as_mut() }));
        let orig = ManuallyDrop::new(orig);
        MappedRwLockWriteGuard {
            data,
            inner_lock: orig.inner_lock,
            poison_flag: orig.poison_flag,
            poison: orig.poison.clone(),
            _variance: PhantomData,
        }
    }

    /// Makes a [`MappedRwLockWriteGuard`] for a component of the borrowed data.
    /// The original guard is returned as an `Err(...)` if the closure returns
    /// `None`.
    ///
    /// The `RwLock` is already locked for writing, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `MappedRwLockWriteGuard::try_map(...)`. A method would interfere with
    /// methods of the same name on the contents of the `MappedRwLockWriteGuard`
    /// used through `Deref`.
    ///
    /// # Panics
    ///
    /// If the closure panics, the guard will be dropped (unlocked) and the RwLock will be poisoned.
    #[doc(alias = "filter_map")]
    pub fn try_map<U, F>(mut orig: Self, f: F) -> Result<MappedRwLockWriteGuard<'a, U, L>, Self>
    where
        F: FnOnce(&mut T) -> Option<&mut U>,
        U: ?Sized,
    {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the
        // reference passed to it. If the closure panics, the guard will be dropped.
        match f(unsafe { orig.data.as_mut() }) {
            Some(data) => {
                let data = NonNull::from(data);
                let orig = ManuallyDrop::new(orig);
                Ok(MappedRwLockWriteGuard {
                    data,
                    inner_lock: orig.inner_lock,
                    poison_flag: orig.poison_flag,
                    poison: orig.poison.clone(),
                    _variance: PhantomData,
                })
            },
            None => Err(orig),
        }
    }
}

#[cfg(all(test, not(target_os = "emscripten")))]
mod tests;

use core::{
    cell::UnsafeCell,
    fmt,
    marker::PhantomData,
    mem::{self, ManuallyDrop},
    ops::{Deref, DerefMut},
    ptr::NonNull,
    time::Duration,
};

use crate::{
    psp_fw_select,
    sync::{
        nonpoison::{TryLockError, TryLockResult},
        RawMutex, RawMutexTimed,
    },
    sys::sync as sys,
    time::Instant,
};

type DefaultMutex = psp_fw_select! {
    ..270 => sys::SemaMutex,
    270..395 => sys::Mutex,
    395.. => cfg_select! {
        // pbp => sys::LwMutex,
        _ => sys::Mutex,
    },
    _ => sys::SpinMutex,
};

/// A mutual exclusion primitive useful for protecting shared data
///
/// This mutex will block threads waiting for the lock to become available. The
/// mutex can be created via a [`new`] constructor. Each mutex has a type parameter
/// which represents the data that it is protecting. The data can only be accessed
/// through the RAII guards returned from [`lock`] and [`try_lock`], which
/// guarantees that the data is only ever accessed when the mutex is locked.
///
/// # Poisoning
///
/// The mutexes in this module don't implement a strategy called "poisoning" where a
/// mutex is considered poisoned whenever a thread panics while holding the
/// mutex. For a version with this strategy, checks the [`poison::Mutex`].
///
/// [`poison::Mutex`]: crate::sync::poison::Mutex
/// [`new`]: Self::new
/// [`lock`]: Self::lock
/// [`try_lock`]: Self::try_lock
///
/// # Examples
///
/// ```
/// use std::{
///     sync::{mpsc::channel, Arc},
///     thread,
/// };
///
/// use pspsdk::sync::nonpoison::Mutex;
///
/// const N: usize = 10;
///
/// // Spawn a few threads to increment a shared variable (non-atomically), and
/// // let the main thread know once all increments are done.
/// //
/// // Here we're using an Arc to share memory among threads, and the data inside
/// // the Arc is protected with a mutex.
/// let data = Arc::new(Mutex::new(0));
///
/// let (tx, rx) = channel();
/// for _ in 0..N {
///     let (data, tx) = (Arc::clone(&data), tx.clone());
///     thread::spawn(move || {
///         // The shared state can only be accessed once the lock is held.
///         // Our non-atomic increment is safe because we're the only thread
///         // which can access the shared state when the lock is held.
///         //
///         // We unwrap() the return value to assert that we are not expecting
///         // threads to ever fail while holding the lock.
///         let mut data = data.lock();
///         *data += 1;
///         if *data == N {
///             tx.send(()).unwrap();
///         }
///         // the lock is unlocked here when `data` goes out of scope.
///     });
/// }
///
/// rx.recv().unwrap();
/// ```
///
/// To unlock a mutex guard sooner than the end of the enclosing scope,
/// either create an inner scope or drop the guard manually.
///
/// ```
/// use std::{sync::Arc, thread};
///
/// use pspsdk::sync::nonpoison::Mutex;
///
/// const N: usize = 3;
///
/// let data_mutex = Arc::new(Mutex::new(vec![1, 2, 3, 4]));
/// let res_mutex = Arc::new(Mutex::new(0));
///
/// let mut threads = Vec::with_capacity(N);
/// (0..N).for_each(|_| {
///     let data_mutex_clone = Arc::clone(&data_mutex);
///     let res_mutex_clone = Arc::clone(&res_mutex);
///
///     threads.push(thread::spawn(move || {
///         // Here we use a block to limit the lifetime of the lock guard.
///         let result = {
///             let mut data = data_mutex_clone.lock();
///             // This is the result of some important and long-ish work.
///             let result = data.iter().fold(0, |acc, x| acc + x * 2);
///             data.push(result);
///             result
///             // The mutex guard gets dropped here, together with any other values
///             // created in the critical section.
///         };
///         // The guard created here is a temporary dropped at the end of the statement, i.e.
///         // the lock would not remain being held even if the thread did some additional work.
///         *res_mutex_clone.lock() += result;
///     }));
/// });
///
/// let mut data = data_mutex.lock();
/// // This is the result of some important and long-ish work.
/// let result = data.iter().fold(0, |acc, x| acc + x * 2);
/// data.push(result);
/// // We drop the `data` explicitly because it's not necessary anymore and the
/// // thread still has work to do. This allows other threads to start working on
/// // the data immediately, without waiting for the rest of the unrelated work
/// // to be done here.
/// //
/// // It's even more important here than in the threads because we `.join` the
/// // threads after that. If we had not dropped the mutex guard, a thread could
/// // be waiting forever for it, causing a deadlock.
/// // As in the threads, a block could have been used instead of calling the
/// // `drop` function.
/// drop(data);
/// // Here the mutex guard is not assigned to a variable and so, even if the
/// // scope does not end after this line, the mutex is still released: there is
/// // no deadlock.
/// *res_mutex.lock() += result;
///
/// threads
///     .into_iter()
///     .for_each(|thread| thread.join().expect("The thread creating or execution failed !"));
///
/// assert_eq!(*res_mutex.lock(), 800);
/// ```
pub struct Mutex<T: ?Sized, M = DefaultMutex> {
    inner: M,
    data: UnsafeCell<T>,
}

// these are the only places where `T: Send` matters; all other
// functionality works fine on a single thread.
unsafe impl<T: ?Sized + Send, M> Send for Mutex<T, M> {}
unsafe impl<T: ?Sized + Send, M> Sync for Mutex<T, M> {}

impl<T: ?Sized, M> core::panic::UnwindSafe for Mutex<T, M> {}
impl<T: ?Sized, M> core::panic::RefUnwindSafe for Mutex<T, M> {}

/// An RAII implementation of a "scoped lock" of a mutex. When this structure is
/// dropped (falls out of scope), the lock will be unlocked.
///
/// The data protected by the mutex can be accessed through this guard via its
/// [`Deref`] and [`DerefMut`] implementations.
///
/// This structure is created by the [`lock`] and [`try_lock`] methods on
/// [`Mutex`].
///
/// [`lock`]: Mutex::lock
/// [`try_lock`]: Mutex::try_lock
#[must_use = "if unused the Mutex will immediately unlock"]
#[clippy::has_significant_drop]
pub struct MutexGuard<'a, T: ?Sized + 'a, M: RawMutex = DefaultMutex> {
    lock: &'a Mutex<T, M>,
}

impl<T: ?Sized, M: RawMutex> !Send for MutexGuard<'_, T, M> {}
unsafe impl<T: ?Sized + Sync, M: RawMutex> Sync for MutexGuard<'_, T, M> {}

/// An RAII mutex guard returned by `PoisonMutexGuard::map`, which can point to a
/// subfield of the protected data. When this structure is dropped (falls out
/// of scope), the lock will be unlocked.
///
/// The main difference between `MappedMutexGuard` and [`MutexGuard`] is that the
/// former cannot be used with [`Condvar`], since that
/// could introduce soundness issues if the locked object is modified by another
/// thread while the `Mutex` is unlocked.
///
/// The data protected by the mutex can be accessed through this guard via its
/// [`Deref`] and [`DerefMut`] implementations.
///
/// This structure is created by the [`map`] and [`try_map`] methods on
/// [`MutexGuard`].
///
/// [`map`]: MutexGuard::map
/// [`try_map`]: MutexGuard::try_map
/// [`Condvar`]: crate::sync::nonpoison::Condvar
#[must_use = "if unused the Mutex will immediately unlock"]
#[clippy::has_significant_drop]
pub struct MappedMutexGuard<'a, T: ?Sized + 'a, M: RawMutex = DefaultMutex> {
    // NB: we use a pointer instead of `&'a mut T` to avoid `noalias` violations, because a
    // `MappedMutexGuard` argument doesn't hold uniqueness for its whole scope, only until it
    // drops. `NonNull` is covariant over `T`, so we add a `PhantomData<&'a mut T>` field
    // below for the correct variance over `T` (invariance).
    data: NonNull<T>,
    inner: &'a M,
    _variance: PhantomData<&'a mut T>,
}

impl<T: ?Sized, M: RawMutex> !Send for MappedMutexGuard<'_, T, M> {}
unsafe impl<T: ?Sized + Sync, M: RawMutex> Sync for MappedMutexGuard<'_, T, M> {}

impl<T> Mutex<T> {
    /// Creates a new mutex in an unlocked state ready for use.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::nonpoison::Mutex;
    ///
    /// let mutex = Mutex::new(0);
    /// ```
    #[inline]
    pub const fn new(t: T) -> Mutex<T> {
        Mutex {
            inner: DefaultMutex::new(),
            data: UnsafeCell::new(t),
        }
    }
}

impl<T, M: RawMutex> Mutex<T, M> {
    /// Creates a new mutex in an unlocked state ready for use using the specified mutex.
    #[inline]
    pub const fn new_with(value: T, raw_mutex: M) -> Self {
        Self {
            inner: raw_mutex,
            data: UnsafeCell::new(value),
        }
    }

    /// Returns the contained value by cloning it.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::nonpoison::Mutex;
    ///
    /// let mut mutex = Mutex::new(7);
    ///
    /// assert_eq!(mutex.get_cloned(), 7);
    /// ```
    pub fn get_cloned(&self) -> T
    where
        T: Clone,
    {
        let guard = self.lock();
        (*guard).clone()
    }

    /// Sets the contained value.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::nonpoison::Mutex;
    ///
    /// let mut mutex = Mutex::new(7);
    ///
    /// assert_eq!(mutex.get_cloned(), 7);
    /// mutex.set(11);
    /// assert_eq!(mutex.get_cloned(), 11);
    /// ```
    pub fn set(&self, value: T) {
        if mem::needs_drop::<T>() {
            // If the contained value has non-trivial destructor, we
            // call that destructor after the lock being released.
            let res = self.replace(value);
            drop(res);
        } else {
            let mut guard = self.lock();
            *guard = value;
        }
    }

    /// Replaces the contained value with `value`, and returns the old contained value.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::nonpoison::Mutex;
    ///
    /// let mut mutex = Mutex::new(7);
    ///
    /// assert_eq!(mutex.replace(11), 7);
    /// assert_eq!(mutex.get_cloned(), 11);
    /// ```
    pub fn replace(&self, value: T) -> T {
        let mut guard = self.lock();
        mem::replace(&mut *guard, value)
    }
}

impl<T: ?Sized, M: RawMutex> Mutex<T, M> {
    /// Acquires a mutex, blocking the current thread until it is able to do so.
    ///
    /// This function will block the local thread until it is available to acquire
    /// the mutex. Upon returning, the thread is the only thread with the lock
    /// held. An RAII guard is returned to allow scoped unlock of the lock. When
    /// the guard goes out of scope, the mutex will be unlocked.
    ///
    /// The exact behavior on locking a mutex in the thread which already holds
    /// the lock is left unspecified. However, this function will not return on
    /// the second call (it might panic or deadlock, for example).
    ///
    /// Attempts to lock a mutex in the thread which already holds the lock will result in a
    /// deadlock.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{sync::Arc, thread};
    ///
    /// use pspsdk::sync::nonpoison::Mutex;
    ///
    /// let mutex = Arc::new(Mutex::new(0));
    /// let c_mutex = Arc::clone(&mutex);
    ///
    /// thread::spawn(move || {
    ///     *c_mutex.lock() = 10;
    /// })
    /// .join()
    /// .expect("thread::spawn failed");
    /// assert_eq!(*mutex.lock(), 10);
    /// ```
    pub fn lock(&self) -> MutexGuard<'_, T, M> {
        self.inner.lock();
        unsafe { MutexGuard::new(self) }
    }

    /// Attempts to acquire this lock.
    ///
    /// If the lock could not be acquired at this time, then [`Err`] is returned.
    /// Otherwise, an RAII guard is returned. The lock will be unlocked when the
    /// guard is dropped.
    ///
    /// This function does not block.
    ///
    /// # Errors
    ///
    /// If the mutex could not be acquired because it is already locked, then
    /// this call will return the [`WouldBlock`] error.
    ///
    /// [`WouldBlock`]: TryLockError::WouldBlock
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{sync::Arc, thread};
    ///
    /// use pspsdk::sync::nonpoison::Mutex;
    ///
    /// let mutex = Arc::new(Mutex::new(0));
    /// let c_mutex = Arc::clone(&mutex);
    ///
    /// thread::spawn(move || {
    ///     let mut lock = c_mutex.try_lock();
    ///     if let Ok(ref mut mutex) = lock {
    ///         **mutex = 10;
    ///     } else {
    ///         println!("try_lock failed");
    ///     }
    /// })
    /// .join()
    /// .expect("thread::spawn failed");
    /// assert_eq!(*mutex.lock(), 10);
    /// ```
    pub fn try_lock(&self) -> TryLockResult<MutexGuard<'_, T, M>> {
        if self.inner.try_lock() {
            Ok(unsafe { MutexGuard::new(self) })
        } else {
            Err(TryLockError::WouldBlock)
        }
    }

    /// Consumes this mutex, returning the underlying data.
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::nonpoison::Mutex;
    ///
    /// let mutex = Mutex::new(0);
    /// assert_eq!(mutex.into_inner(), 0);
    /// ```
    pub fn into_inner(self) -> T
    where
        T: Sized,
    {
        self.data.into_inner()
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the `Mutex` mutably, no actual locking needs to
    /// take place -- the mutable borrow statically guarantees no locks exist.
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::nonpoison::Mutex;
    ///
    /// let mut mutex = Mutex::new(0);
    /// *mutex.get_mut() = 10;
    /// assert_eq!(*mutex.lock(), 10);
    /// ```
    pub fn get_mut(&mut self) -> &mut T {
        self.data.get_mut()
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

    /// Acquires the mutex and provides mutable access to the underlying data by passing
    /// a mutable reference to the given closure.
    ///
    /// This method acquires the lock, calls the provided closure with a mutable reference
    /// to the data, and returns the result of the closure. The lock is released after
    /// the closure completes, even if it panics.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::nonpoison::Mutex;
    ///
    /// let mutex = Mutex::new(2);
    ///
    /// let result = mutex.with_mut(|data| {
    ///     *data += 3;
    ///
    ///     *data + 5
    /// });
    ///
    /// assert_eq!(*mutex.lock(), 5);
    /// assert_eq!(result, 10);
    /// ```
    pub fn with_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(&mut self.lock())
    }
}

impl<T: ?Sized, M: RawMutexTimed> Mutex<T, M> {
    /// Attempts to acquire this lock until a timeout is reached.
    ///
    /// If the lock could not be acquired before the timeout expired, then
    /// `None` is returned. Otherwise, an RAII guard is returned. The lock will
    /// be unlocked when the guard is dropped.
    #[inline]
    #[track_caller]
    pub fn try_lock_for(&self, timeout: Duration) -> Option<MutexGuard<'_, T, M>> {
        if self.inner.try_lock_for(timeout) {
            // SAFETY: The lock is held, as required.
            Some(unsafe { MutexGuard::new(self) })
        } else {
            None
        }
    }

    /// Attempts to acquire this lock until a timeout is reached.
    ///
    /// If the lock could not be acquired before the timeout expired, then
    /// `None` is returned. Otherwise, an RAII guard is returned. The lock will
    /// be unlocked when the guard is dropped.
    #[inline]
    #[track_caller]
    pub fn try_lock_until(&self, timeout: Instant) -> Option<MutexGuard<'_, T, M>> {
        if self.inner.try_lock_until(timeout) {
            // SAFETY: The lock is held, as required.
            Some(unsafe { MutexGuard::new(self) })
        } else {
            None
        }
    }
}

impl<T> From<T> for Mutex<T> {
    /// Creates a new mutex in an unlocked state ready for use.
    /// This is equivalent to [`Mutex::new`].
    fn from(t: T) -> Self {
        Mutex::new(t)
    }
}

impl<T> Default for Mutex<T>
where
    T: Default,
{
    /// Creates a `Mutex<T>`, with the `Default` value for T.
    fn default() -> Mutex<T> {
        Mutex::new(Default::default())
    }
}

impl<T: ?Sized + fmt::Debug, M: RawMutex> fmt::Debug for Mutex<T, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("Mutex");
        match self.try_lock() {
            Ok(guard) => {
                d.field("data", &&*guard);
            },
            Err(TryLockError::WouldBlock) => {
                d.field("data", &format_args!("<locked>"));
            },
        }
        d.finish_non_exhaustive()
    }
}

impl<'mutex, T: ?Sized, M: RawMutex> MutexGuard<'mutex, T, M> {
    unsafe fn new(lock: &'mutex Mutex<T, M>) -> MutexGuard<'mutex, T, M> {
        MutexGuard { lock }
    }
}

impl<T: ?Sized, M: RawMutex> Deref for MutexGuard<'_, T, M> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized, M: RawMutex> DerefMut for MutexGuard<'_, T, M> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T: ?Sized, M: RawMutex> Drop for MutexGuard<'_, T, M> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.lock.inner.unlock();
        }
    }
}

impl<T, M> fmt::Debug for MutexGuard<'_, T, M>
where
    T: ?Sized + fmt::Debug,
    M: RawMutex,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T, M> fmt::Display for MutexGuard<'_, T, M>
where
    T: ?Sized + fmt::Display,
    M: RawMutex,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

pub fn guard_lock<'a, T, M>(guard: &MutexGuard<'a, T, M>) -> &'a M
where
    T: ?Sized,
    M: RawMutex,
{
    &guard.lock.inner
}

impl<'a, T: ?Sized, M: RawMutex> MutexGuard<'a, T, M> {
    /// Makes a [`MappedMutexGuard`] for a component of the borrowed data, e.g.
    /// an enum variant.
    ///
    /// The `Mutex` is already locked, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `MutexGuard::map(...)`. A method would interfere with methods of the
    /// same name on the contents of the `MutexGuard` used through `Deref`.
    pub fn map<U, F>(orig: Self, f: F) -> MappedMutexGuard<'a, U, M>
    where
        F: FnOnce(&mut T) -> &mut U,
        U: ?Sized,
    {
        // SAFETY: the conditions of `MutexGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the
        // reference passed to it. If the closure panics, the guard will be dropped.
        let data = NonNull::from(f(unsafe { &mut *orig.lock.data.get() }));
        let orig = ManuallyDrop::new(orig);
        MappedMutexGuard {
            data,
            inner: &orig.lock.inner,
            _variance: PhantomData,
        }
    }

    /// Makes a [`MappedMutexGuard`] for a component of the borrowed data. The
    /// original guard is returned as an `Err(...)` if the closure returns
    /// `None`.
    ///
    /// The `Mutex` is already locked, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `MutexGuard::try_map(...)`. A method would interfere with methods of the
    /// same name on the contents of the `MutexGuard` used through `Deref`.
    #[doc(alias = "filter_map")]
    pub fn try_map<U, F>(orig: Self, f: F) -> Result<MappedMutexGuard<'a, U, M>, Self>
    where
        F: FnOnce(&mut T) -> Option<&mut U>,
        U: ?Sized,
    {
        // SAFETY: the conditions of `MutexGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the
        // reference passed to it. If the closure panics, the guard will be dropped.
        match f(unsafe { &mut *orig.lock.data.get() }) {
            Some(data) => {
                let data = NonNull::from(data);
                let orig = ManuallyDrop::new(orig);
                Ok(MappedMutexGuard {
                    data,
                    inner: &orig.lock.inner,
                    _variance: PhantomData,
                })
            },
            None => Err(orig),
        }
    }
}

impl<T: ?Sized, M: RawMutex> Deref for MappedMutexGuard<'_, T, M> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.data.as_ref() }
    }
}

impl<T: ?Sized, M: RawMutex> DerefMut for MappedMutexGuard<'_, T, M> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.data.as_mut() }
    }
}

impl<T: ?Sized, M: RawMutex> Drop for MappedMutexGuard<'_, T, M> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.inner.unlock();
        }
    }
}

impl<T, M> fmt::Debug for MappedMutexGuard<'_, T, M>
where
    T: ?Sized + fmt::Debug,
    M: RawMutex,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T, M> fmt::Display for MappedMutexGuard<'_, T, M>
where
    T: ?Sized + fmt::Display,
    M: RawMutex,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<'a, T: ?Sized, M: RawMutex> MappedMutexGuard<'a, T, M> {
    /// Makes a [`MappedMutexGuard`] for a component of the borrowed data, e.g.
    /// an enum variant.
    ///
    /// The `Mutex` is already locked, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `MappedMutexGuard::map(...)`. A method would interfere with methods of the
    /// same name on the contents of the `MutexGuard` used through `Deref`.
    pub fn map<U, F>(mut orig: Self, f: F) -> MappedMutexGuard<'a, U, M>
    where
        F: FnOnce(&mut T) -> &mut U,
        U: ?Sized,
    {
        // SAFETY: the conditions of `MutexGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the
        // reference passed to it. If the closure panics, the guard will be dropped.
        let data = NonNull::from(f(unsafe { orig.data.as_mut() }));
        let orig = ManuallyDrop::new(orig);
        MappedMutexGuard {
            data,
            inner: orig.inner,
            _variance: PhantomData,
        }
    }

    /// Makes a [`MappedMutexGuard`] for a component of the borrowed data. The
    /// original guard is returned as an `Err(...)` if the closure returns
    /// `None`.
    ///
    /// The `Mutex` is already locked, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `MappedMutexGuard::try_map(...)`. A method would interfere with methods of the
    /// same name on the contents of the `MutexGuard` used through `Deref`.
    #[doc(alias = "filter_map")]
    pub fn try_map<U, F>(mut orig: Self, f: F) -> Result<MappedMutexGuard<'a, U, M>, Self>
    where
        F: FnOnce(&mut T) -> Option<&mut U>,
        U: ?Sized,
    {
        // SAFETY: the conditions of `MutexGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the
        // reference passed to it. If the closure panics, the guard will be dropped.
        match f(unsafe { orig.data.as_mut() }) {
            Some(data) => {
                let data = NonNull::from(data);
                let orig = ManuallyDrop::new(orig);
                Ok(MappedMutexGuard {
                    data,
                    inner: orig.inner,
                    _variance: PhantomData,
                })
            },
            None => Err(orig),
        }
    }
}

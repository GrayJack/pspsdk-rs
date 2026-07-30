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
    sync::{
        poison::{self, LockResult, TryLockError, TryLockResult},
        RawMutex, RawMutexTimed,
    },
    sys::sync as sys,
    time::Instant,
};

use super::PoisonError;

type DefaultMutex = cfg_select! {
    feature = "kernel" => sys::Mutex,
    pbp => sys::LwMutex,
    _ => sys::SemaMutex,
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
/// The mutexes in this module implement a strategy called "poisoning" where a
/// mutex is considered poisoned whenever a thread panics while holding the
/// mutex. Once a mutex is poisoned, all other threads are unable to access the
/// data by default as it is likely tainted (some invariant is not being
/// upheld).
///
/// For a mutex, this means that the [`lock`] and [`try_lock`] methods return a
/// [`Result`] which indicates whether a mutex has been poisoned or not. Most
/// usage of a mutex will simply [`unwrap()`] these results, propagating panics
/// among threads to ensure that a possibly invalid invariant is not witnessed.
///
/// A poisoned mutex, however, does not prevent all access to the underlying
/// data. The [`PoisonError`] type has an [`into_inner`] method which will return
/// the guard that would have otherwise been returned on a successful lock. This
/// allows access to the data, despite the lock being poisoned.
///
/// [`new`]: Self::new
/// [`lock`]: Self::lock
/// [`try_lock`]: Self::try_lock
/// [`unwrap()`]: Result::unwrap
/// [`PoisonError`]: super::PoisonError
/// [`into_inner`]: super::PoisonError::into_inner
///
/// # Examples
///
/// ```
/// use std::{
///     sync::{mpsc::channel, Arc},
///     thread,
/// };
///
/// use pspsdk::sync::poison::Mutex;
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
/// To recover from a poisoned mutex:
///
/// ```
/// use std::{sync::Arc, thread};
///
/// use pspsdk::sync::poison::Mutex;
///
/// let lock = Arc::new(Mutex::new(0_u32));
/// let lock2 = Arc::clone(&lock);
///
/// let _ = thread::spawn(move || -> () {
///     // This thread will acquire the mutex first, unwrapping the result of
///     // `lock` because the lock has not been poisoned.
///     let _guard = lock2.lock();
///
///     // This panic while holding the lock (`_guard` is in scope) will poison
///     // the mutex.
///     panic!();
/// })
/// .join();
///
/// // The lock is poisoned by this point, but the returned result can be
/// // pattern matched on to return the underlying guard on both branches.
/// let mut guard = match lock.lock_checked() {
///     Ok(guard) => guard,
///     Err(poisoned) => poisoned.into_inner(),
/// };
///
/// *guard += 1;
/// ```
///
/// To unlock a mutex guard sooner than the end of the enclosing scope,
/// either create an inner scope or drop the guard manually.
///
/// ```
/// use std::{sync::Arc, thread};
///
/// use pspsdk::sync::poison::Mutex;
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
    poison: poison::Flag,
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
    poison: poison::Guard,
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
/// [`Condvar`]: crate::sync::poison::Condvar
#[must_use = "if unused the Mutex will immediately unlock"]
#[clippy::has_significant_drop]
pub struct MappedMutexGuard<'a, T: ?Sized + 'a, M: RawMutex = DefaultMutex> {
    // NB: we use a pointer instead of `&'a mut T` to avoid `noalias` violations, because a
    // `MappedMutexGuard` argument doesn't hold uniqueness for its whole scope, only until it
    // drops. `NonNull` is covariant over `T`, so we add a `PhantomData<&'a mut T>` field
    // below for the correct variance over `T` (invariance).
    data: NonNull<T>,
    inner: &'a M,
    poison_flag: &'a poison::Flag,
    poison: poison::Guard,
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
    /// use pspsdk::sync::poison::Mutex;
    ///
    /// let mutex = Mutex::new(0);
    /// ```
    #[inline]
    pub const fn new(t: T) -> Mutex<T> {
        Mutex {
            inner: DefaultMutex::new(),
            poison: poison::Flag::new(),
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
            poison: poison::Flag::new(),
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
    /// use pspsdk::sync::poison::Mutex;
    ///
    /// let mut mutex = Mutex::new(7);
    ///
    /// assert_eq!(mutex.get_cloned().unwrap(), 7);
    /// ```
    pub fn get_cloned(&self) -> Result<T, PoisonError<()>>
    where
        T: Clone,
    {
        match self.lock_checked() {
            Ok(guard) => Ok((*guard).clone()),
            Err(_) => Err(PoisonError::new(())),
        }
    }

    /// Sets the contained value.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error containing the provided `value` instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::poison::Mutex;
    ///
    /// let mut mutex = Mutex::new(7);
    ///
    /// assert_eq!(mutex.get_cloned().unwrap(), 7);
    /// mutex.set(11).unwrap();
    /// assert_eq!(mutex.get_cloned().unwrap(), 11);
    /// ```
    pub fn set(&self, value: T) -> Result<(), PoisonError<T>> {
        if mem::needs_drop::<T>() {
            // If the contained value has non-trivial destructor, we
            // call that destructor after the lock being released.
            self.replace(value).map(drop)
        } else {
            match self.lock_checked() {
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
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error containing the provided `value` instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::poison::Mutex;
    ///
    /// let mut mutex = Mutex::new(7);
    ///
    /// assert_eq!(mutex.replace(11).unwrap(), 7);
    /// assert_eq!(mutex.get_cloned().unwrap(), 11);
    /// ```
    pub fn replace(&self, value: T) -> LockResult<T> {
        match self.lock_checked() {
            Ok(mut guard) => Ok(mem::replace(&mut *guard, value)),
            Err(_) => Err(PoisonError::new(value)),
        }
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
    /// # Panics
    ///
    /// This function might panic when called if the lock is already held by
    /// the current thread or the lock is in a poisoned state.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{sync::Arc, thread};
    ///
    /// use pspsdk::sync::poison::Mutex;
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
    #[track_caller]
    pub fn lock(&self) -> MutexGuard<'_, T, M> {
        self.inner.lock();
        match unsafe { MutexGuard::new(self) } {
            Ok(guard) => guard,
            Err(err) => panic!("{err}"),
        }
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
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return the [`Poisoned`] error if the mutex would
    /// otherwise be acquired. An acquired lock guard will be contained
    /// in the returned error.
    ///
    /// If the mutex could not be acquired because it is already locked, then
    /// this call will return the [`WouldBlock`] error.
    ///
    /// [`Poisoned`]: TryLockError::Poisoned
    /// [`WouldBlock`]: TryLockError::WouldBlock
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{sync::Arc, thread};
    ///
    /// use pspsdk::sync::poison::Mutex;
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
            Ok(unsafe { MutexGuard::new(self)? })
        } else {
            Err(TryLockError::WouldBlock)
        }
    }

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
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error once the mutex is acquired. The acquired
    /// mutex guard will be contained in the returned error.
    ///
    /// # Panics
    ///
    /// This function might panic when called if the lock is already held by
    /// the current thread.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{sync::Arc, thread};
    ///
    /// use pspsdk::sync::poison::Mutex;
    ///
    /// let mutex = Arc::new(Mutex::new(0));
    /// let c_mutex = Arc::clone(&mutex);
    ///
    /// thread::spawn(move || {
    ///     *c_mutex.lock_checked().unwrap() = 10;
    /// })
    /// .join()
    /// .expect("thread::spawn failed");
    /// assert_eq!(*mutex.lock(), 10);
    /// ```
    pub fn lock_checked(&self) -> LockResult<MutexGuard<'_, T, M>> {
        self.inner.lock();
        unsafe { MutexGuard::new(self) }
    }

    /// Same as [`lock`](Self::lock), but ignoring poison state instead of panicking.
    ///
    /// # Panics
    ///
    /// This function might panic when called if the lock is already held by
    /// the current thread.
    pub fn lock_unchecked(&self) -> MutexGuard<'_, T, M> {
        self.inner.lock();
        match unsafe { MutexGuard::new(self) } {
            Ok(guard) => guard,
            Err(err) => err.into_inner(),
        }
    }

    /// Same as [`try_lock`](Self::try_lock), but ignoring poison state.
    ///
    /// # Errors
    ///
    /// If the mutex could not be acquired because it is already locked, then
    /// this call will return the [`WouldBlock`] error.
    ///
    /// [`WouldBlock`]: TryLockError::WouldBlock
    pub fn try_unchecked(&self) -> TryLockResult<MutexGuard<'_, T, M>> {
        match self.try_lock() {
            Ok(guard) => Ok(guard),
            Err(TryLockError::Poisoned(err)) => Ok(err.into_inner()),
            Err(err) => Err(err),
        }
    }

    /// Determines whether the mutex is poisoned.
    ///
    /// If another thread is active, the mutex can still become poisoned at any
    /// time. You should not trust a `false` value for program correctness
    /// without additional synchronization.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{sync::Arc, thread};
    ///
    /// use pspsdk::sync::poison::Mutex;
    ///
    /// let mutex = Arc::new(Mutex::new(0));
    /// let c_mutex = Arc::clone(&mutex);
    ///
    /// let _ = thread::spawn(move || {
    ///     let _lock = c_mutex.lock();
    ///     panic!(); // the mutex gets poisoned
    /// })
    /// .join();
    /// assert_eq!(mutex.is_poisoned(), true);
    /// ```
    #[inline]
    pub fn is_poisoned(&self) -> bool {
        self.poison.get()
    }

    /// Clear the poisoned state from a mutex.
    ///
    /// If the mutex is poisoned, it will remain poisoned until this function is called. This
    /// allows recovering from a poisoned state and marking that it has recovered. For example, if
    /// the value is overwritten by a known-good value, then the mutex can be marked as
    /// un-poisoned. Or possibly, the value could be inspected to determine if it is in a
    /// consistent state, and if so the poison is removed.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{sync::Arc, thread};
    ///
    /// use pspsdk::sync::poison::Mutex;
    ///
    /// let mutex = Arc::new(Mutex::new(0));
    /// let c_mutex = Arc::clone(&mutex);
    ///
    /// let _ = thread::spawn(move || {
    ///     let _lock = c_mutex.lock();
    ///     panic!(); // the mutex gets poisoned
    /// })
    /// .join();
    ///
    /// assert_eq!(mutex.is_poisoned(), true);
    /// let x = mutex.lock_checked().unwrap_or_else(|mut e| {
    ///     **e.get_mut() = 1;
    ///     mutex.clear_poison();
    ///     e.into_inner()
    /// });
    /// assert_eq!(mutex.is_poisoned(), false);
    /// assert_eq!(*x, 1);
    /// ```
    #[inline]
    pub fn clear_poison(&self) {
        self.poison.clear();
    }

    /// Consumes this mutex, returning the underlying data.
    ///
    /// # Panics
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will panic instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::poison::Mutex;
    ///
    /// let mutex = Mutex::new(0);
    /// assert_eq!(mutex.into_inner(), 0);
    /// ```
    #[track_caller]
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

    /// Consumes this mutex, returning the underlying data, explicitly cheking poison state.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error containing the the underlying data
    /// instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::poison::Mutex;
    ///
    /// let mutex = Mutex::new(0);
    /// assert_eq!(mutex.into_inner_checked().unwrap(), 0);
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
    /// Since this call borrows the `Mutex` mutably, no actual locking needs to
    /// take place -- the mutable borrow statically guarantees no locks exist.
    ///
    /// # Panics
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will panic instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::poison::Mutex;
    ///
    /// let mut mutex = Mutex::new(0);
    /// *mutex.get_mut() = 10;
    /// assert_eq!(*mutex.lock(), 10);
    /// ```
    #[track_caller]
    pub fn get_mut(&mut self) -> &mut T {
        let data = self.data.get_mut();
        match poison::map_result(self.poison.borrow(), |()| data) {
            Ok(data) => data,
            Err(err) => panic!("{err}"),
        }
    }

    /// Returns a mutable reference to the underlying data, explicitly cheking poison state.
    ///
    /// Since this call borrows the `Mutex` mutably, no actual locking needs to
    /// take place -- the mutable borrow statically guarantees no locks exist.
    ///
    /// # Errors
    ///
    /// If another user of this mutex panicked while holding the mutex, then
    /// this call will return an error containing a mutable reference to the
    /// underlying data instead.
    ///
    /// # Examples
    /// ```
    /// use pspsdk::sync::poison::Mutex;
    ///
    /// let mut mutex = Mutex::new(0);
    /// *mutex.get_mut_checked().unwrap() = 10;
    /// assert_eq!(*mutex.lock(), 10);
    /// ```
    pub fn get_mut_checked(&mut self) -> LockResult<&mut T> {
        let data = self.data.get_mut();
        poison::map_result(self.poison.borrow(), |()| data)
    }

    /// Consumes this mutex ignoring the poison state, returning the underlying data.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::poison::Mutex;
    ///
    /// let mutex = Mutex::new(0);
    /// assert_eq!(mutex.into_inner_unchecked(), 0);
    /// ```
    pub fn into_inner_unchecked(self) -> T
    where
        T: Sized,
    {
        self.data.into_inner()
    }

    /// Returns a mutable reference to the underlying data, ignoring poison state.
    ///
    /// Since this call borrows the `Mutex` mutably, no actual locking needs to
    /// take place -- the mutable borrow statically guarantees no locks exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::poison::Mutex;
    ///
    /// let mut mutex = Mutex::new(0);
    /// *mutex.get_mut_unchecked() = 10;
    /// assert_eq!(*mutex.lock(), 10);
    /// ```
    pub fn get_mut_unchecked(&mut self) -> &mut T {
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
            unsafe { MutexGuard::new(self).ok() }
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
            unsafe { MutexGuard::new(self).ok() }
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
        crate::println!("GOT HERE 1");
        let mut d = f.debug_struct("Mutex");
        match self.try_lock() {
            Ok(guard) => {
                crate::println!("GOT HERE 2");
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

impl<'mutex, T: ?Sized, M: RawMutex> MutexGuard<'mutex, T, M> {
    unsafe fn new(lock: &'mutex Mutex<T, M>) -> LockResult<MutexGuard<'mutex, T, M>> {
        poison::map_result(lock.poison.guard(), |guard| MutexGuard {
            lock,
            poison: guard,
        })
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
            self.lock.poison.done(&self.poison);
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

pub fn guard_poison<'a, T, M>(guard: &MutexGuard<'a, T, M>) -> &'a poison::Flag
where
    T: ?Sized,
    M: RawMutex,
{
    &guard.lock.poison
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
            poison_flag: &orig.lock.poison,
            poison: orig.poison.clone(),
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
                    poison_flag: &orig.lock.poison,
                    poison: orig.poison.clone(),
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
            self.poison_flag.done(&self.poison);
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
            poison_flag: orig.poison_flag,
            poison: orig.poison.clone(),
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
                    poison_flag: orig.poison_flag,
                    poison: orig.poison.clone(),
                    _variance: PhantomData,
                })
            },
            None => Err(orig),
        }
    }
}

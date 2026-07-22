use core::{fmt, ops::DerefMut, time::Duration};

use crate::{
    sync::{
        nonpoison::{mutex, mutex::MutexGuard},
        RawMutex, WaitTimeoutResult,
    },
    sys::sync as sys,
    time::Instant,
};

/// A Condition Variable
///
/// Condition variables represent the ability to block a thread such that it
/// consumes no CPU time while waiting for an event to occur. Condition
/// variables are typically associated with a boolean predicate (a condition)
/// and a mutex. The predicate is always verified inside of the mutex before
/// determining that a thread must block.
///
/// Functions in this module will block the current **thread** of execution.
/// Note that any attempt to use multiple mutexes on the same condition
/// variable may result in a runtime panic.
///
/// # Examples
///
/// ```
/// use alloc::sync::Arc;
///
/// use pspsdk::{
///     sync::nonpoison::{Condvar, Mutex},
///     thread,
/// };
///
/// let pair = Arc::new((Mutex::new(false), Condvar::new()));
/// let pair2 = Arc::clone(&pair);
///
/// // Inside of our lock, spawn a new thread, and then wait for it to start.
/// thread::spawn(move || {
///     let (lock, cvar) = &*pair2;
///     let mut started = lock.lock();
///     *started = true;
///     // We notify the condvar that the value has changed.
///     cvar.notify_one();
/// });
///
/// // Wait for the thread to start up.
/// let (lock, cvar) = &*pair;
/// let mut started = lock.lock();
/// while !*started {
///     started = cvar.wait(started);
/// }
/// ```
pub struct Condvar {
    inner: sys::Condvar,
}

impl core::panic::UnwindSafe for Condvar {}
impl core::panic::RefUnwindSafe for Condvar {}

impl Condvar {
    /// Creates a new condition variable which is ready to be waited on and
    /// notified.
    ///
    /// # Examples
    ///
    /// ```
    /// use pspsdk::sync::nonpoison::Condvar;
    ///
    /// let condvar = Condvar::new();
    /// ```
    #[must_use]
    #[inline]
    pub const fn new() -> Condvar {
        Condvar {
            inner: sys::Condvar::new(),
        }
    }

    /// Wakes up one blocked thread on this condvar.
    ///
    /// If there is a blocked thread on this condition variable, then it will
    /// be woken up from its call to [`wait`] or [`wait_timeout`]. Calls to
    /// `notify_one` are not buffered in any way.
    ///
    /// To wake up all threads, see [`notify_all`].
    ///
    /// [`wait`]: Self::wait
    /// [`wait_timeout`]: Self::wait_timeout
    /// [`notify_all`]: Self::notify_all
    ///
    /// # Examples
    ///
    /// ```
    /// use alloc::sync::Arc;
    ///
    /// use pspsdk::{
    ///     sync::nonpoison::{Condvar, Mutex},
    ///     thread,
    /// };
    ///
    /// let pair = Arc::new((Mutex::new(false), Condvar::new()));
    /// let pair2 = Arc::clone(&pair);
    ///
    /// thread::spawn(move || {
    ///     let (lock, cvar) = &*pair2;
    ///     let mut started = lock.lock();
    ///     *started = true;
    ///     // We notify the condvar that the value has changed.
    ///     cvar.notify_one();
    /// });
    ///
    /// // Wait for the thread to start up.
    /// let (lock, cvar) = &*pair;
    /// let mut started = lock.lock();
    /// // As long as the value inside the `Mutex<bool>` is `false`, we wait.
    /// while !*started {
    ///     started = cvar.wait(started);
    /// }
    /// ```
    pub fn notify_one(&self) {
        self.inner.notify_one()
    }

    /// Wakes up all blocked threads on this condvar.
    ///
    /// This method will ensure that any current waiters on the condition
    /// variable are awoken. Calls to `notify_all()` are not buffered in any
    /// way.
    ///
    /// To wake up only one thread, see [`notify_one`].
    ///
    /// [`notify_one`]: Self::notify_one
    ///
    /// # Examples
    ///
    /// ```
    /// use alloc::sync::Arc;
    ///
    /// use pspsdk::{
    ///     sync::nonpoison::{Condvar, Mutex},
    ///     thread,
    /// };
    ///
    /// let pair = Arc::new((Mutex::new(false), Condvar::new()));
    /// let pair2 = Arc::clone(&pair);
    ///
    /// thread::spawn(move || {
    ///     let (lock, cvar) = &*pair2;
    ///     let mut started = lock.lock();
    ///     *started = true;
    ///     // We notify the condvar that the value has changed.
    ///     cvar.notify_all();
    /// });
    ///
    /// // Wait for the thread to start up.
    /// let (lock, cvar) = &*pair;
    /// let mut started = lock.lock();
    /// // As long as the value inside the `Mutex<bool>` is `false`, we wait.
    /// while !*started {
    ///     started = cvar.wait(started);
    /// }
    /// ```
    pub fn notify_all(&self) {
        self.inner.notify_all()
    }
}

impl Condvar {
    /// Blocks the current thread until this condition variable receives a
    /// notification.
    ///
    /// This function will atomically unlock the mutex specified (represented by
    /// `guard`) and block the current thread. This means that any calls
    /// to [`notify_one`] or [`notify_all`] which happen logically after the
    /// mutex is unlocked are candidates to wake this thread up. When this
    /// function call returns, the lock specified will have been re-acquired.
    ///
    /// Note that this function is susceptible to spurious wakeups. Condition
    /// variables normally have a boolean predicate associated with them, and
    /// the predicate must always be checked each time this function returns to
    /// protect against spurious wakeups.
    ///
    /// [`notify_one`]: Self::notify_one
    /// [`notify_all`]: Self::notify_all
    /// [`Mutex`]: super::Mutex
    ///
    /// # Examples
    ///
    /// ```
    /// use alloc::sync::Arc;
    ///
    /// use pspsdk::{
    ///     sync::nonpoison::{Condvar, Mutex},
    ///     thread,
    /// };
    ///
    /// let pair = Arc::new((Mutex::new(false), Condvar::new()));
    /// let pair2 = Arc::clone(&pair);
    ///
    /// thread::spawn(move || {
    ///     let (lock, cvar) = &*pair2;
    ///     let mut started = lock.lock();
    ///     *started = true;
    ///     // We notify the condvar that the value has changed.
    ///     cvar.notify_one();
    /// });
    ///
    /// // Wait for the thread to start up.
    /// let (lock, cvar) = &*pair;
    /// let mut started = lock.lock();
    /// // As long as the value inside the `Mutex<bool>` is `false`, we wait.
    /// while !*started {
    ///     cvar.wait(&mut started);
    /// }
    /// ```
    pub fn wait<'a, T, M: RawMutex>(&self, guard: &mut MutexGuard<'a, T, M>) {
        let lock = mutex::guard_lock(guard);
        unsafe { self.inner.wait(lock) };
    }

    /// Blocks the current thread until this condition variable receives a
    /// notification and the provided condition is false.
    ///
    /// This function will atomically unlock the mutex specified (represented by
    /// `guard`) and block the current thread. This means that any calls
    /// to [`notify_one`] or [`notify_all`] which happen logically after the
    /// mutex is unlocked are candidates to wake this thread up. When this
    /// function call returns, the lock specified will have been re-acquired.
    ///
    /// [`notify_one`]: Self::notify_one
    /// [`notify_all`]: Self::notify_all
    /// [`Mutex`]: super::Mutex
    ///
    /// # Examples
    ///
    /// ```
    /// use alloc::sync::Arc;
    ///
    /// use pspsdk::{
    ///     sync::nonpoison::{Condvar, Mutex},
    ///     thread,
    /// };
    ///
    /// let pair = Arc::new((Mutex::new(true), Condvar::new()));
    /// let pair2 = Arc::clone(&pair);
    ///
    /// thread::spawn(move || {
    ///     let (lock, cvar) = &*pair2;
    ///     let mut pending = lock.lock();
    ///     *pending = false;
    ///     // We notify the condvar that the value has changed.
    ///     cvar.notify_one();
    /// });
    ///
    /// // Wait for the thread to start up.
    /// let (lock, cvar) = &*pair;
    /// // As long as the value inside the `Mutex<bool>` is `true`, we wait.
    /// let mut guard = lock.lock();
    /// cvar.wait_while(&mut guard, |pending| *pending);
    /// ```
    pub fn wait_while<'a, T, F, M>(&self, guard: &mut MutexGuard<'a, T, M>, mut condition: F)
    where
        F: FnMut(&mut T) -> bool,
        M: RawMutex,
    {
        while condition(&mut *guard) {
            self.wait(guard);
        }
    }

    /// Waits on this condition variable for a notification, timing out after a
    /// specified duration.
    ///
    /// The semantics of this function are equivalent to [`wait`] except that
    /// the thread will be blocked for roughly no longer than `dur`. This
    /// method should not be used for precise timing due to anomalies such as
    /// preemption or platform differences that might not cause the maximum
    /// amount of time waited to be precisely `dur`.
    ///
    /// Note that the best effort is made to ensure that the time waited is
    /// measured with a monotonic clock, and not affected by the changes made to
    /// the system time. This function is susceptible to spurious wakeups.
    /// Condition variables normally have a boolean predicate associated with
    /// them, and the predicate must always be checked each time this function
    /// returns to protect against spurious wakeups. Additionally, it is
    /// typically desirable for the timeout to not exceed some duration in
    /// spite of spurious wakes, thus the sleep-duration is decremented by the
    /// amount slept. Alternatively, use the `wait_timeout_while` method
    /// to wait with a timeout while a predicate is true.
    ///
    /// The returned [`WaitTimeoutResult`] value indicates if the timeout is
    /// known to have elapsed.
    ///
    /// Like [`wait`], the lock specified will be re-acquired when this function
    /// returns, regardless of whether the timeout elapsed or not.
    ///
    /// [`wait`]: Self::wait
    /// [`wait_timeout_while`]: Self::wait_timeout_while
    ///
    /// # Examples
    ///
    /// ```
    /// use alloc::sync::Arc;
    ///
    /// use pspsdk::{
    ///     sync::nonpoison::{Condvar, Mutex},
    ///     thread,
    ///     time::Duration,
    /// };
    ///
    /// let pair = Arc::new((Mutex::new(false), Condvar::new()));
    /// let pair2 = Arc::clone(&pair);
    ///
    /// thread::spawn(move || {
    ///     let (lock, cvar) = &*pair2;
    ///     let mut started = lock.lock();
    ///     *started = true;
    ///     // We notify the condvar that the value has changed.
    ///     cvar.notify_one();
    /// });
    ///
    /// // wait for the thread to start up
    /// let (lock, cvar) = &*pair;
    /// let mut started = lock.lock();
    /// // as long as the value inside the `Mutex<bool>` is `false`, we wait
    /// loop {
    ///     let result = cvar.wait_timeout(&mut started, Duration::from_millis(10));
    ///     // 10 milliseconds have passed, or maybe the value changed!
    ///     if *started == true {
    ///         // We received the notification and the value has been updated, we can leave.
    ///         break;
    ///     }
    /// }
    /// ```
    pub fn wait_timeout<'a, T, M: RawMutex>(
        &self, guard: &mut MutexGuard<'a, T, M>, dur: Duration,
    ) -> WaitTimeoutResult {
        let success = unsafe {
            let lock = mutex::guard_lock(guard);
            self.inner.wait_timeout(lock, dur)
        };
        WaitTimeoutResult(!success)
    }

    /// Waits on this condition variable for a notification, timing out after a
    /// specified duration.
    ///
    /// The semantics of this function are equivalent to [`wait_while`] except
    /// that the thread will be blocked for roughly no longer than `dur`. This
    /// method should not be used for precise timing due to anomalies such as
    /// preemption or platform differences that might not cause the maximum
    /// amount of time waited to be precisely `dur`.
    ///
    /// Note that the best effort is made to ensure that the time waited is
    /// measured with a monotonic clock, and not affected by the changes made to
    /// the system time.
    ///
    /// The returned [`WaitTimeoutResult`] value indicates if the timeout is
    /// known to have elapsed without the condition being met.
    ///
    /// Like [`wait_while`], the lock specified will be re-acquired when this
    /// function returns, regardless of whether the timeout elapsed or not.
    ///
    /// [`wait_while`]: Self::wait_while
    /// [`wait_timeout`]: Self::wait_timeout
    ///
    /// # Examples
    ///
    /// ```
    /// use alloc::sync::Arc;
    ///
    /// use pspsdk::{
    ///     sync::nonpoison::{Condvar, Mutex},
    ///     thread,
    ///     time::Duration,
    /// };
    ///
    /// let pair = Arc::new((Mutex::new(true), Condvar::new()));
    /// let pair2 = Arc::clone(&pair);
    ///
    /// thread::spawn(move || {
    ///     let (lock, cvar) = &*pair2;
    ///     let mut pending = lock.lock();
    ///     *pending = false;
    ///     // We notify the condvar that the value has changed.
    ///     cvar.notify_one();
    /// });
    ///
    /// // wait for the thread to start up
    /// let (lock, cvar) = &*pair;
    /// let mut guard = lock.lock();
    /// let result =
    ///     cvar.wait_timeout_while(&mut guard, Duration::from_millis(100), |&mut pending| pending);
    /// if result.timed_out() {
    ///     // timed-out without the condition ever evaluating to false.
    /// }
    /// // access the locked mutex via result.0
    /// ```
    pub fn wait_timeout_while<'a, T, F, M>(
        &self, guard: &mut MutexGuard<'a, T, M>, dur: Duration, mut condition: F,
    ) -> WaitTimeoutResult
    where
        F: FnMut(&mut T) -> bool,
        M: RawMutex,
    {
        let start = Instant::now();

        while condition(guard.deref_mut()) {
            let timeout = match dur.checked_sub(start.elapsed()) {
                Some(timeout) => timeout,
                None => return WaitTimeoutResult(true),
            };

            self.wait_timeout(guard, timeout);
        }

        WaitTimeoutResult(false)
    }
}

impl fmt::Debug for Condvar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Condvar").finish_non_exhaustive()
    }
}

impl Default for Condvar {
    /// Creates a `Condvar` which is ready to be waited on and notified.
    fn default() -> Self {
        Condvar::new()
    }
}

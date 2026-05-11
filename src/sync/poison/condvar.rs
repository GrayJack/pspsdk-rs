use core::{fmt, time::Duration};

use crate::{
    sync::{
        poison::{mutex, mutex::MutexGuard, LockResult, PoisonError},
        RawMutex,
    },
    sys::sync as sys,
    time::Instant,
};


/// A type indicating whether a timed wait on a condition variable returned
/// due to a time out or not.
///
/// It is returned by the [`wait_timeout`] method.
///
/// [`wait_timeout`]: Condvar::wait_timeout
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct WaitTimeoutResult(bool);

impl WaitTimeoutResult {
    /// Returns `true` if the wait was known to have timed out.
    ///
    /// # Examples
    ///
    /// This example spawns a thread which will sleep 20 milliseconds before
    /// updating a boolean value and then notifying the condvar.
    ///
    /// The main thread will wait with a 10 millisecond timeout on the condvar
    /// and will leave the loop upon timeout.
    ///
    /// ```
    /// use std::{sync::Arc, thread, time::Duration};
    ///
    /// use pspsdk::sync::poison::{Condvar, Mutex};
    ///
    /// let pair = Arc::new((Mutex::new(false), Condvar::new()));
    /// let pair2 = Arc::clone(&pair);
    ///
    /// thread::spawn(move || {
    ///     let (lock, cvar) = &*pair2;
    ///
    ///     // Let's wait 20 milliseconds before notifying the condvar.
    ///     thread::sleep(Duration::from_millis(20));
    ///
    ///     let mut started = lock.lock();
    ///     // We update the boolean value.
    ///     *started = true;
    ///     cvar.notify_one();
    /// });
    ///
    /// // Wait for the thread to start up.
    /// let (lock, cvar) = &*pair;
    /// loop {
    ///     // Let's put a timeout on the condvar's wait.
    ///     let result = cvar.wait_timeout(lock.lock(), Duration::from_millis(10));
    ///     // 10 milliseconds have passed.
    ///     if result.1.timed_out() {
    ///         // timed out now and we can leave.
    ///         break;
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn timed_out(&self) -> bool {
        self.0
    }
}

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
/// use std::{sync::Arc, thread};
///
/// use pspsdk::sync::poison::{Condvar, Mutex};
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
    /// use pspsdk::sync::poison::Condvar;
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
    /// use std::{sync::Arc, thread};
    ///
    /// use pspsdk::sync::poison::{Condvar, Mutex};
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
    /// use std::{sync::Arc, thread};
    ///
    /// use pspsdk::sync::poison::{Condvar, Mutex};
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
    /// # Panics
    ///
    /// This function may [`panic!`] if it is used with more than one mutex
    /// over time or if the mutex is poisoned.
    ///
    /// [`notify_one`]: Self::notify_one
    /// [`notify_all`]: Self::notify_all
    /// [`Mutex`]: super::Mutex
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{sync::Arc, thread};
    ///
    /// use pspsdk::sync::poison::{Condvar, Mutex};
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
    pub fn wait<'a, T, M: RawMutex>(&self, guard: MutexGuard<'a, T, M>) -> MutexGuard<'a, T, M> {
        let poisoned = unsafe {
            let lock = mutex::guard_lock(&guard);
            self.inner.wait(lock);
            mutex::guard_poison(&guard).get()
        };
        if poisoned {
            panic!("{}", PoisonError::new(guard))
        } else {
            guard
        }
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
    /// # Panics
    ///
    /// This function may [`panic!`] if it is used with more than one mutex
    /// over time or if the mutex is poisoned.
    ///
    /// [`notify_one`]: Self::notify_one
    /// [`notify_all`]: Self::notify_all
    /// [`Mutex`]: super::Mutex
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{sync::Arc, thread};
    ///
    /// use pspsdk::sync::poison::{Condvar, Mutex};
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
    /// let _guard = cvar.wait_while(lock.lock(), |pending| *pending);
    /// ```
    pub fn wait_while<'a, T, F, M>(
        &self, mut guard: MutexGuard<'a, T, M>, mut condition: F,
    ) -> MutexGuard<'a, T, M>
    where
        F: FnMut(&mut T) -> bool,
        M: RawMutex,
    {
        while condition(&mut *guard) {
            guard = self.wait(guard);
        }
        guard
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
    /// # Panics
    ///
    /// This function may [`panic!`] if it is used with more than one mutex
    /// over time or if the mutex is poisoned.
    ///
    /// [`wait`]: Self::wait
    /// [`wait_timeout_while`]: Self::wait_timeout_while
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{sync::Arc, thread, time::Duration};
    ///
    /// use pspsdk::sync::poison::{Condvar, Mutex};
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
    ///     let result = cvar.wait_timeout(started, Duration::from_millis(10));
    ///     // 10 milliseconds have passed, or maybe the value changed!
    ///     started = result.0;
    ///     if *started == true {
    ///         // We received the notification and the value has been updated, we can leave.
    ///         break;
    ///     }
    /// }
    /// ```
    pub fn wait_timeout<'a, T, M: RawMutex>(
        &self, guard: MutexGuard<'a, T, M>, dur: Duration,
    ) -> (MutexGuard<'a, T, M>, WaitTimeoutResult) {
        let (poisoned, result) = unsafe {
            let lock = mutex::guard_lock(&guard);
            let success = self.inner.wait_timeout(lock, dur);
            (mutex::guard_poison(&guard).get(), WaitTimeoutResult(!success))
        };
        if poisoned {
            panic!("{}", PoisonError::new((guard, result)))
        } else {
            (guard, result)
        }
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
    /// # Panics
    ///
    /// This function may [`panic!`] if it is used with more than one mutex
    /// over time or if the mutex is poisoned.
    ///
    /// [`wait_while`]: Self::wait_while
    /// [`wait_timeout`]: Self::wait_timeout
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{sync::Arc, thread, time::Duration};
    ///
    /// use pspsdk::sync::poison::{Condvar, Mutex};
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
    /// let result =
    ///     cvar.wait_timeout_while(lock.lock(), Duration::from_millis(100), |&mut pending| pending);
    /// if result.1.timed_out() {
    ///     // timed-out without the condition ever evaluating to false.
    /// }
    /// // access the locked mutex via result.0
    /// ```
    pub fn wait_timeout_while<'a, T, F, M>(
        &self, mut guard: MutexGuard<'a, T, M>, dur: Duration, mut condition: F,
    ) -> (MutexGuard<'a, T, M>, WaitTimeoutResult)
    where
        F: FnMut(&mut T) -> bool,
        M: RawMutex,
    {
        let start = Instant::now();
        loop {
            if !condition(&mut *guard) {
                return (guard, WaitTimeoutResult(false));
            }
            let timeout = match dur.checked_sub(start.elapsed()) {
                Some(timeout) => timeout,
                None => return (guard, WaitTimeoutResult(true)),
            };
            guard = self.wait_timeout(guard, timeout).0;
        }
    }
}

impl Condvar {
    /// Blocks the current thread until this condition variable receives a
    /// notification checking and returning poisoning status.
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
    /// # Errors
    ///
    /// This function will return an error if the mutex being waited on is
    /// poisoned when this thread re-acquires the lock. For more information,
    /// see information about [poisoning] on the [`Mutex`] type.
    ///
    /// # Panics
    ///
    /// This function may [`panic!`] if it is used with more than one mutex
    /// over time.
    ///
    /// [`notify_one`]: Self::notify_one
    /// [`notify_all`]: Self::notify_all
    /// [poisoning]: super::Mutex#poisoning
    /// [`Mutex`]: super::Mutex
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{sync::Arc, thread};
    ///
    /// use pspsdk::sync::poison::{Condvar, Mutex};
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
    ///     started = cvar.wait_checked(started).unwrap();
    /// }
    /// ```
    pub fn wait_checked<'a, T, M: RawMutex>(
        &self, guard: MutexGuard<'a, T, M>,
    ) -> LockResult<MutexGuard<'a, T, M>> {
        let poisoned = unsafe {
            let lock = mutex::guard_lock(&guard);
            self.inner.wait(lock);
            mutex::guard_poison(&guard).get()
        };
        if poisoned {
            Err(PoisonError::new(guard))
        } else {
            Ok(guard)
        }
    }

    /// Blocks the current thread until this condition variable receives a
    /// notification and the provided condition is false checking and returning poisoning status.
    ///
    /// This function will atomically unlock the mutex specified (represented by
    /// `guard`) and block the current thread. This means that any calls
    /// to [`notify_one`] or [`notify_all`] which happen logically after the
    /// mutex is unlocked are candidates to wake this thread up. When this
    /// function call returns, the lock specified will have been re-acquired.
    ///
    /// # Errors
    ///
    /// This function will return an error if the mutex being waited on is
    /// poisoned when this thread re-acquires the lock. For more information,
    /// see information about [poisoning] on the [`Mutex`] type.
    ///
    /// [`notify_one`]: Self::notify_one
    /// [`notify_all`]: Self::notify_all
    /// [poisoning]: super::Mutex#poisoning
    /// [`Mutex`]: super::Mutex
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{sync::Arc, thread};
    ///
    /// use pspsdk::sync::poison::{Condvar, Mutex};
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
    /// let _guard = cvar.wait_while_checked(lock.lock(), |pending| *pending).unwrap();
    /// ```
    pub fn wait_while_checked<'a, T, F, M>(
        &self, mut guard: MutexGuard<'a, T, M>, mut condition: F,
    ) -> LockResult<MutexGuard<'a, T, M>>
    where
        F: FnMut(&mut T) -> bool,
        M: RawMutex,
    {
        while condition(&mut *guard) {
            guard = self.wait_checked(guard)?;
        }
        Ok(guard)
    }

    /// Waits on this condition variable for a notification, timing out after a
    /// specified duration checking and returning poisoning status.
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
    /// use std::{sync::Arc, thread, time::Duration};
    ///
    /// use pspsdk::sync::poison::{Condvar, Mutex};
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
    ///     let result = cvar.wait_timeout_checked(started, Duration::from_millis(10)).unwrap();
    ///     // 10 milliseconds have passed, or maybe the value changed!
    ///     started = result.0;
    ///     if *started == true {
    ///         // We received the notification and the value has been updated, we can leave.
    ///         break;
    ///     }
    /// }
    /// ```
    pub fn wait_timeout_checked<'a, T, M: RawMutex>(
        &self, guard: MutexGuard<'a, T, M>, dur: Duration,
    ) -> LockResult<(MutexGuard<'a, T, M>, WaitTimeoutResult)> {
        let (poisoned, result) = unsafe {
            let lock = mutex::guard_lock(&guard);
            let success = self.inner.wait_timeout(lock, dur);
            (mutex::guard_poison(&guard).get(), WaitTimeoutResult(!success))
        };
        if poisoned {
            Err(PoisonError::new((guard, result)))
        } else {
            Ok((guard, result))
        }
    }

    /// Waits on this condition variable for a notification, timing out after a
    /// specified duration, checking and returning poisoning status.
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
    /// use std::{sync::Arc, thread, time::Duration};
    ///
    /// use pspsdk::sync::poison::{Condvar, Mutex};
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
    /// let result = cvar
    ///     .wait_timeout_while_checked(lock.lock(), Duration::from_millis(100), |&mut pending| pending)
    ///     .unwrap();
    /// if result.1.timed_out() {
    ///     // timed-out without the condition ever evaluating to false.
    /// }
    /// // access the locked mutex via result.0
    /// ```
    pub fn wait_timeout_while_checked<'a, T, F, M>(
        &self, mut guard: MutexGuard<'a, T, M>, dur: Duration, mut condition: F,
    ) -> LockResult<(MutexGuard<'a, T, M>, WaitTimeoutResult)>
    where
        F: FnMut(&mut T) -> bool,
        M: RawMutex,
    {
        let start = Instant::now();
        loop {
            if !condition(&mut *guard) {
                return Ok((guard, WaitTimeoutResult(false)));
            }
            let timeout = match dur.checked_sub(start.elapsed()) {
                Some(timeout) => timeout,
                None => return Ok((guard, WaitTimeoutResult(true))),
            };
            guard = self.wait_timeout_checked(guard, timeout)?.0;
        }
    }
}

impl Condvar {
    /// Blocks the current thread until this condition variable receives a
    /// notification, completely ignoring poisoning status.
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
    /// # Panics
    ///
    /// This function may [`panic!`] if it is used with more than one mutex
    /// over time.
    ///
    /// [`notify_one`]: Self::notify_one
    /// [`notify_all`]: Self::notify_all
    /// [`Mutex`]: super::Mutex
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{sync::Arc, thread};
    ///
    /// use pspsdk::sync::poison::{Condvar, Mutex};
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
    ///     started = cvar.wait_unchecked(started);
    /// }
    /// ```
    pub fn wait_unchecked<'a, T, M: RawMutex>(
        &self, guard: MutexGuard<'a, T, M>,
    ) -> MutexGuard<'a, T, M> {
        unsafe {
            let lock = mutex::guard_lock(&guard);
            self.inner.wait(lock);
        };
        guard
    }

    /// Blocks the current thread until this condition variable receives a
    /// notification and the provided condition is false, completely ignoring poisoning status.
    ///
    /// This function will atomically unlock the mutex specified (represented by
    /// `guard`) and block the current thread. This means that any calls
    /// to [`notify_one`] or [`notify_all`] which happen logically after the
    /// mutex is unlocked are candidates to wake this thread up. When this
    /// function call returns, the lock specified will have been re-acquired.
    ///
    /// # Panics
    ///
    /// This function may [`panic!`] if it is used with more than one mutex
    /// over time.
    ///
    /// [`notify_one`]: Self::notify_one
    /// [`notify_all`]: Self::notify_all
    /// [`Mutex`]: super::Mutex
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{sync::Arc, thread};
    ///
    /// use pspsdk::sync::poison::{Condvar, Mutex};
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
    /// let _guard = cvar.wait_while_unchecked(lock.lock(), |pending| *pending);
    /// ```
    pub fn wait_while_unchecked<'a, T, F, M>(
        &self, mut guard: MutexGuard<'a, T, M>, mut condition: F,
    ) -> MutexGuard<'a, T, M>
    where
        F: FnMut(&mut T) -> bool,
        M: RawMutex,
    {
        while condition(&mut *guard) {
            guard = self.wait(guard);
        }
        guard
    }

    /// Waits on this condition variable for a notification, timing out after a
    /// specified duration, completely ignoring poisoning status.
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
    /// # Panics
    ///
    /// This function may [`panic!`] if it is used with more than one mutex
    /// over time or if the mutex is poisoned.
    ///
    /// [`wait`]: Self::wait
    /// [`wait_timeout_while`]: Self::wait_timeout_while
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{sync::Arc, thread, time::Duration};
    ///
    /// use pspsdk::sync::poison::{Condvar, Mutex};
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
    ///     let result = cvar.wait_timeout_unchecked(started, Duration::from_millis(10));
    ///     // 10 milliseconds have passed, or maybe the value changed!
    ///     started = result.0;
    ///     if *started == true {
    ///         // We received the notification and the value has been updated, we can leave.
    ///         break;
    ///     }
    /// }
    /// ```
    pub fn wait_timeout_unchecked<'a, T, M: RawMutex>(
        &self, guard: MutexGuard<'a, T, M>, dur: Duration,
    ) -> (MutexGuard<'a, T, M>, WaitTimeoutResult) {
        unsafe {
            let lock = mutex::guard_lock(&guard);
            let success = self.inner.wait_timeout(lock, dur);
            (guard, WaitTimeoutResult(!success))
        }
    }

    /// Waits on this condition variable for a notification, timing out after a
    /// specified duration, completely ignoring poisoning status.
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
    /// # Panics
    ///
    /// This function may [`panic!`] if it is used with more than one mutex
    /// over time.
    ///
    /// [`wait_while`]: Self::wait_while
    /// [`wait_timeout`]: Self::wait_timeout
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{sync::Arc, thread, time::Duration};
    ///
    /// use pspsdk::sync::poison::{Condvar, Mutex};
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
    /// let result = cvar.wait_timeout_while_unchecked(
    ///     lock.lock(),
    ///     Duration::from_millis(100),
    ///     |&mut pending| pending,
    /// );
    /// if result.1.timed_out() {
    ///     // timed-out without the condition ever evaluating to false.
    /// }
    /// // access the locked mutex via result.0
    /// ```
    pub fn wait_timeout_while_unchecked<'a, T, F, M>(
        &self, mut guard: MutexGuard<'a, T, M>, dur: Duration, mut condition: F,
    ) -> (MutexGuard<'a, T, M>, WaitTimeoutResult)
    where
        F: FnMut(&mut T) -> bool,
        M: RawMutex,
    {
        let start = Instant::now();
        loop {
            if !condition(&mut *guard) {
                return (guard, WaitTimeoutResult(false));
            }
            let timeout = match dur.checked_sub(start.elapsed()) {
                Some(timeout) => timeout,
                None => return (guard, WaitTimeoutResult(true)),
            };
            guard = self.wait_timeout(guard, timeout).0;
        }
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

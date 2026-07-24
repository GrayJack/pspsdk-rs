//! Useful synchronization primitives.
//!
//! ## The need for synchronization
//!
//! Conceptually, a Rust program is a series of operations which will
//! be executed on a computer. The timeline of events happening in the
//! program is consistent with the order of the operations in the code.
//!
//! Consider the following code, operating on some global static variables:
//!
//! ```rust
//! // FIXME(static_mut_refs): use raw pointers instead of references
//! #![allow(static_mut_refs)]
//!
//! static mut A: u32 = 0;
//! static mut B: u32 = 0;
//! static mut C: u32 = 0;
//!
//! fn main() {
//!     unsafe {
//!         A = 3;
//!         B = 4;
//!         A = A + B;
//!         C = B;
//!         println!("{A} {B} {C}");
//!         C = A;
//!     }
//! }
//! ```
//!
//! It appears as if some variables stored in memory are changed, an addition
//! is performed, result is stored in `A` and the variable `C` is
//! modified twice.
//!
//! When only a single thread is involved, the results are as expected:
//! the line `7 4 4` gets printed.
//!
//! As for what happens behind the scenes, when optimizations are enabled the
//! final generated machine code might look very different from the code:
//!
//! - The first store to `C` might be moved before the store to `A` or `B`, _as if_ we had written
//!   `C = 4; A = 3; B = 4`.
//!
//! - Assignment of `A + B` to `A` might be removed, since the sum can be stored in a temporary
//!   location until it gets printed, with the global variable never getting updated.
//!
//! - The final result could be determined just by looking at the code at compile time, so [constant
//!   folding] might turn the whole block into a simple `println!("7 4 4")`.
//!
//! The compiler is allowed to perform any combination of these
//! optimizations, as long as the final optimized code, when executed,
//! produces the same results as the one without optimizations.
//!
//! Due to the [concurrency] involved in modern computers, assumptions
//! about the program's execution order are often wrong. Access to
//! global variables can lead to nondeterministic results, **even if**
//! compiler optimizations are disabled, and it is **still possible**
//! to introduce synchronization bugs.
//!
//! Note that thanks to Rust's safety guarantees, accessing global (static)
//! variables requires `unsafe` code, assuming we don't use any of the
//! synchronization primitives in this module.
//!
//! [constant folding]: https://en.wikipedia.org/wiki/Constant_folding
//! [concurrency]: https://en.wikipedia.org/wiki/Concurrency_(computer_science)
//!
//! ## Out-of-order execution
//!
//! Instructions can execute in a different order from the one we define, due to
//! various reasons:
//!
//! - The **compiler** reordering instructions: If the compiler can issue an instruction at an
//!   earlier point, it will try to do so. For example, it might hoist memory loads at the top of a
//!   code block, so that the CPU can start [prefetching] the values from memory.
//!
//!   In single-threaded scenarios, this can cause issues when writing
//!   signal handlers or certain kinds of low-level code.
//!   Use [compiler fences] to prevent this reordering.
//!
//! - A **single processor** executing instructions [out-of-order]: Modern CPUs are capable of
//!   [superscalar] execution, i.e., multiple instructions might be executing at the same time, even
//!   though the machine code describes a sequential process.
//!
//!   This kind of reordering is handled transparently by the CPU.
//!
//! - A **multiprocessor** system executing multiple hardware threads at the same time: In
//!   multi-threaded scenarios, you can use two kinds of primitives to deal with synchronization:
//!   - [memory fences] to ensure memory accesses are made visible to other CPUs in the right order.
//!   - [atomic operations] to ensure simultaneous access to the same memory location doesn't lead
//!     to undefined behavior.
//!
//! [prefetching]: https://en.wikipedia.org/wiki/Cache_prefetching
//! [compiler fences]: core::sync::atomic::compiler_fence
//! [out-of-order]: https://en.wikipedia.org/wiki/Out-of-order_execution
//! [superscalar]: https://en.wikipedia.org/wiki/Superscalar_processor
//! [memory fences]: core::sync::atomic::fence
//! [atomic operations]: core::sync::atomic
//!
//! ## Higher-level synchronization objects
//!
//! Most of the low-level synchronization primitives are quite error-prone and
//! inconvenient to use, which is why the standard library also exposes some
//! higher-level synchronization objects.
//!
//! These abstractions can be built out of lower-level primitives.
//! For efficiency, the sync objects in the standard library are usually
//! implemented with help from the operating system's kernel, which is
//! able to reschedule the threads while they are blocked on acquiring
//! a lock.
//!
//! The following is an overview of the available synchronization
//! objects:
//!
//! - [`Arc`]: Atomically Reference-Counted pointer, which can be used in multithreaded environments
//!   to prolong the lifetime of some data until all the threads have finished using it.
//!
//! - [`Barrier`]: Ensures multiple threads will wait for each other to reach a point in the
//!   program, before continuing execution all together.
//!
//! - [`Condvar`]: Condition Variable, providing the ability to block a thread while waiting for an
//!   event to occur.
// ! - [`mpsc`]: Multi-producer, single-consumer queues, used for message-based communication. Can
// !   provide a lightweight inter-thread synchronisation mechanism, at the cost of some extra
// !   memory.
// !
// ! - [`mpmc`]: Multi-producer, multi-consumer queues, used for message-based communication. Can
// !   provide a lightweight inter-thread synchronisation mechanism, at the cost of some extra
// !   memory.
//! - [`Mutex`]: Mutual Exclusion mechanism, which ensures that at most one thread at a time is able
//!   to access some data.
//!
//! - [`Once`]: Used for a thread-safe, one-time global initialization routine. Mostly useful for
//!   implementing other types like [`OnceLock`].
//!
//! - [`OnceLock`]: Used for thread-safe, one-time initialization of a variable, with potentially
//!   different initializers based on the caller.
//!
//! - [`LazyLock`]: Used for thread-safe, one-time initialization of a variable, using one nullary
//!   initializer function provided at creation.
//!
//! - [`RwLock`]: Provides a mutual exclusion mechanism which allows multiple readers at the same
//!   time, while allowing only one writer at a time. In some cases, this can be more efficient than
//!   a mutex.
//!
//! [`Arc`]: alloc::sync::Arc
//! [`Barrier`]: crate::sync::nonpoison::Barrier
//! [`Condvar`]: crate::sync::nonpoison::Condvar
//! [`mpmc`]: crate::sync::mpmc
//! [`mpsc`]: crate::sync::mpsc
//! [`Mutex`]: crate::sync::nonpoison::Mutex
//! [`Once`]: crate::sync::nonpoison::Once
//! [`OnceLock`]: crate::sync::nonpoison::OnceLock
//! [`RwLock`]: crate::sync::nonpoison::RwLock
//! [`LazyLock`]: crate::sync::nonpoison::LazyLock
use core::fmt;

// Re-exports to make things easier
pub use alloc::sync::{Arc, Weak};
pub use core::sync::atomic;

pub mod nonpoison;
pub mod poison;

pub use nonpoison::{
    Barrier, BarrierWaitResult, Condvar, LazyLock, MappedMutexGuard, MappedRwLockReadGuard,
    MappedRwLockWriteGuard, Mutex, MutexGuard, OnceLock, ReentrantLock, ReentrantLockGuard, RwLock,
    RwLockReadGuard, RwLockWriteGuard,
};
pub use poison::{Once, PoisonError};

mod traits;

pub use traits::{RawMutex, RawMutexTimed, RawRwLock, RawRwLockTimed};

pub(crate) enum ExclusiveState {
    Incomplete,
    Poisoned,
    Complete,
}

/// State yielded to [`Once::call_once_force()`]’s closure parameter. The state
/// can be used to query the poison status of the [`Once`].
///
/// [`Once`]: crate::sync::nonpoison::Once
/// [`Once::call_once_force()`]: crate::sync::nonpoison::Once::call_once_force
pub struct OnceState {
    pub(crate) inner: crate::sys::sync::OnceState,
}

impl OnceState {
    /// Returns `true` if the associated [`Once`] was poisoned prior to the
    /// invocation of the closure passed to [`Once::call_once_force()`].
    ///
    /// # Examples
    ///
    /// A poisoned [`Once`]:
    ///
    /// ```
    /// use std::thread;
    ///
    /// use pspsdk::sync::poison::Once;
    ///
    /// static INIT: Once = Once::new();
    ///
    /// // poison the once
    /// let handle = thread::spawn(|| {
    ///     INIT.call_once(|| panic!());
    /// });
    /// assert!(handle.join().is_err());
    ///
    /// INIT.call_once_force(|state| {
    ///     assert!(state.is_poisoned());
    /// });
    /// ```
    ///
    /// An unpoisoned [`Once`]:
    ///
    /// ```
    /// use pspsdk::sync::poison::Once;
    ///
    /// static INIT: Once = Once::new();
    ///
    /// INIT.call_once_force(|state| {
    ///     assert!(!state.is_poisoned());
    /// });
    /// ```
    ///
    /// [`Once`]: crate::sync::poison::Once
    /// [`Once::call_once_force()`]: crate::sync::poison::Once::call_once_force
    #[inline]
    pub fn is_poisoned(&self) -> bool {
        self.inner.is_poisoned()
    }

    /// Poison the associated [`Once`] without explicitly panicking.
    ///
    /// [`Once`]: crate::sync::poison::once
    // NOTE: This is currently only exposed for `OnceLock`.
    #[inline]
    pub(crate) fn poison(&self) {
        self.inner.poison();
    }
}

impl fmt::Debug for OnceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OnceState").field("poisoned", &self.is_poisoned()).finish()
    }
}

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
    /// use pspsdk::sync::nonpoison::{Condvar, Mutex};
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

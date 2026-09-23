use core::{cell::SyncUnsafeCell, num::NonZero};

/// A unique identifier for a running thread.
///
/// A `ThreadId` is an opaque object that uniquely identifies each thread
/// created during the lifetime of a process. `ThreadId`s are guaranteed not to
/// be reused, even when a thread terminates. `ThreadId`s are under the control
/// of Rust's standard library and there may not be any relationship between
/// `ThreadId` and the underlying platform's notion of a thread identifier --
/// the two concepts cannot, therefore, be used interchangeably. A `ThreadId`
/// can be retrieved from the [`id`] method on a [`Thread`].
///
/// # Examples
///
/// ```
/// use pspsdk::thread;
///
/// let other_thread = thread::spawn(|| thread::current().id());
///
/// let other_thread_id = other_thread.join().unwrap();
/// assert!(thread::current().id() != other_thread_id);
/// ```
///
/// [`Thread`]: super::Thread
/// [`id`]: super::Thread::id
#[derive(Eq, PartialEq, Clone, Copy, Hash, Debug)]
pub struct ThreadId(NonZero<u64>);

impl ThreadId {
    // Generate a new unique thread ID.
    pub(crate) fn new() -> ThreadId {
        #[cold]
        fn exhausted() -> ! {
            panic!("failed to generate unique thread ID: bitspace exhausted")
        }

        // Here we will diverge from std and take advantage we are on a PSP and update the 64bit ID
        // during disabled interrupts. This removes the need to do spin-locks even though the system
        // doesn't have 64bit atomics. If you want to know how Rust STD does, look up for the
        // similar file on Rust source.

        static COUNTER: SyncUnsafeCell<u64> = SyncUnsafeCell::new(0);


        let id = crate::process::with_suspended_interrupts(|| {
            let last = unsafe { COUNTER.get().read_volatile() };

            let id = last.checked_add(1)?;

            unsafe { COUNTER.get().write(id) };
            Some(ThreadId(unsafe { NonZero::new_unchecked(id) }))
        });

        match id {
            Some(id) => id,
            None => exhausted(),
        }
    }

    // #[cfg(any(not(target_thread_local), target_has_atomic = "64"))]
    pub(super) fn from_u64(v: u64) -> Option<ThreadId> {
        NonZero::new(v).map(ThreadId)
    }

    /// This returns a numeric identifier for the thread identified by this
    /// `ThreadId`.
    ///
    /// As noted in the documentation for the type itself, it is essentially an
    /// opaque ID, but is guaranteed to be unique for each thread. The returned
    /// value is entirely opaque -- only equality testing is stable. Note that
    /// it is not guaranteed which values new threads will return, and this may
    /// change across Rust versions.
    #[must_use]
    pub fn as_u64(&self) -> NonZero<u64> {
        self.0
    }
}

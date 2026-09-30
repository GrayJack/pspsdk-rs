#![allow(unused, reason = "FW version dependant")]
use core::{mem::ManuallyDrop, ptr};

use crate::{local_pointer, sys::thread as imp};

use super::{id::ThreadId, main_thread, thread::Thread};

const NONE: *mut () = ptr::null_mut();
const BUSY: *mut () = ptr::without_provenance_mut(1);
const DESTROYED: *mut () = ptr::without_provenance_mut(2);

#[cfg(fw_has_thread_local)]
local_pointer! {
    static CURRENT;
}

/// Persistent storage for the thread ID.
///
/// We store the thread ID so that it never gets destroyed during the lifetime
/// of a thread, either using `#[thread_local]` or multiple `local_pointer!`s.
#[cfg(fw_has_thread_local)]
pub(super) mod id {
    use super::*;

    cfg_select! {
        target_pointer_width = "16" => {
            local_pointer! {
                static ID0;
                static ID16;
                static ID32;
                static ID48;
            }

            pub(super) const CHEAP: bool = false;

            pub(crate) fn get() -> Option<ThreadId> {
                let id0 = ID0.get().addr() as u64;
                let id16 = ID16.get().addr() as u64;
                let id32 = ID32.get().addr() as u64;
                let id48 = ID48.get().addr() as u64;
                ThreadId::from_u64((id48 << 48) + (id32 << 32) + (id16 << 16) + id0)
            }

            pub(super) fn set(id: ThreadId) {
                let val = id.as_u64().get();
                ID0.set(ptr::without_provenance_mut(val as usize));
                ID16.set(ptr::without_provenance_mut((val >> 16) as usize));
                ID32.set(ptr::without_provenance_mut((val >> 32) as usize));
                ID48.set(ptr::without_provenance_mut((val >> 48) as usize));
            }
        },
        target_pointer_width = "32" => {
            local_pointer! {
                static ID0;
                static ID32;
            }

            pub(super) const CHEAP: bool = false;

            pub(crate) fn get() -> Option<ThreadId> {
                let id0 = ID0.get().addr() as u64;
                let id32 = ID32.get().addr() as u64;
                ThreadId::from_u64((id32 << 32) + id0)
            }

            pub(super) fn set(id: ThreadId) {
                let val = id.as_u64().get();
                ID0.set(ptr::without_provenance_mut(val as usize));
                ID32.set(ptr::without_provenance_mut((val >> 32) as usize));
            }
        },
        _ => {
            local_pointer! {
                static ID;
            }

            pub(super) const CHEAP: bool = true;

            pub(crate) fn get() -> Option<ThreadId> {
                let id = ID.get().addr() as u64;
                ThreadId::from_u64(id)
            }

            pub(super) fn set(id: ThreadId) {
                let val = id.as_u64().get();
                ID.set(ptr::without_provenance_mut(val as usize));
            }
        },
    }

    #[inline]
    pub(super) fn get_or_init() -> ThreadId {
        get().unwrap_or_else(
            #[cold]
            || {
                let id = ThreadId::new();
                id::set(id);
                id
            },
        )
    }
}

/// Tries to set the thread handle for the current thread. Fails if a handle was
/// already set or if the thread ID of `thread` would change an already-set ID.
#[cfg(fw_has_thread_local)]
pub(super) fn set_current(thread: Thread) -> Result<(), Thread> {
    if CURRENT.get() != NONE {
        return Err(thread);
    }

    match id::get() {
        Some(id) if id == thread.id() => {},
        None => id::set(thread.id()),
        _ => return Err(thread),
    }

    // Make sure that `crate::rt::thread_cleanup` will be run, which will
    // call `drop_current`.
    crate::sys::thread_local::guard::enable();
    CURRENT.set(thread.into_raw().cast_mut());
    Ok(())
}

/// Gets the unique identifier of the thread which invokes it.
///
/// Calling this function may be more efficient than accessing the current
/// thread id through the current thread handle. i.e. `thread::current().id()`.
///
/// This function will always succeed, will always return the same value for
/// one thread and is guaranteed not to call the global allocator.
///
/// # Examples
///
/// ```
/// use pspsdk::thread;
///
/// let other_thread = thread::spawn(|| thread::current_id());
///
/// let other_thread_id = other_thread.join().unwrap();
/// assert_ne!(thread::current_id(), other_thread_id);
/// ```
#[inline]
#[must_use]
#[cfg(fw_has_thread_local)]
pub fn current_id() -> ThreadId {
    // If accessing the persistent thread ID takes multiple TLS accesses, try
    // to retrieve it from the current thread handle, which will only take one
    // TLS access.
    if !id::CHEAP
        && let Some(id) = try_with_current(|t| t.map(|t| t.id()))
    {
        return id;
    }

    id::get_or_init()
}

/// Gets the OS thread ID of the thread that invokes it, if available. If not, return the Rust
/// thread ID.
///
/// We use a `u64` to all possible platform IDs without excess `cfg`; most use `int`, some use a
/// pointer, and Apple uses `uint64_t`. This is a "best effort" approach for diagnostics and is
/// allowed to fall back to a non-OS ID (such as the Rust thread ID) or a non-unique ID (such as a
/// PID) if the thread ID cannot be retrieved.
#[doc(hidden)]
pub fn current_os_id() -> super::ThreadOsId {
    // imp::current_os_id().unwrap_or_else(|| current_id().as_u64().get())
    imp::current_os_id().unwrap_or(unsafe { super::ThreadOsId::from_raw_unchecked(0) })
}

/// Gets a reference to the handle of the thread that invokes it, if the handle
/// has been initialized.
#[cfg(fw_has_thread_local)]
fn try_with_current<F, R>(f: F) -> R
where
    F: FnOnce(Option<&Thread>) -> R,
{
    let current = CURRENT.get();
    if current > DESTROYED {
        // SAFETY: `Arc` does not contain interior mutability, so it does not
        // matter that the address of the handle might be different depending
        // on where this is called.
        unsafe {
            let current = ManuallyDrop::new(Thread::from_raw(current));
            f(Some(&current))
        }
    } else {
        f(None)
    }
}

/// Run a function with the current thread's name.
///
/// Modulo thread local accesses, this function is safe to call from signal
/// handlers and in similar circumstances where allocations are not possible.
#[cfg(fw_has_thread_local)]
pub(crate) fn with_current_name<F, R>(f: F) -> R
where
    F: FnOnce(Option<&str>) -> R,
{
    try_with_current(|thread| {
        let name = if let Some(thread) = thread {
            // If there is a current thread handle, try to use the name stored
            // there.
            thread.name()
        } else if let Some(main) = main_thread::get()
            && let Some(id) = id::get()
            && id == main
        {
            // The main thread doesn't always have a thread handle, we must
            // identify it through its ID instead. The checks are ordered so
            // that the current ID is only loaded if it is actually needed,
            // since loading it from TLS might need multiple expensive accesses.
            Some("main")
        } else {
            None
        };

        f(name)
    })
}

/// Gets a handle to the thread that invokes it. If the handle stored in thread-
/// local storage was already destroyed, this creates a new unnamed temporary
/// handle to allow thread parking in nearly all situations.
#[cfg(fw_has_thread_local)]
pub(crate) fn current_or_unnamed() -> Thread {
    let current = CURRENT.get();
    if current > DESTROYED {
        unsafe {
            let current = ManuallyDrop::new(Thread::from_raw(current));
            (*current).clone()
        }
    } else if current == DESTROYED {
        let id = id::get_or_init();
        Thread::new_current(id)
    } else {
        init_current(current)
    }
}

/// Gets a handle to the thread that invokes it.
///
/// # Examples
///
/// Getting a handle to the current thread with `thread::current()`:
///
/// ```
/// use pspsdk::thread;
///
/// let handler = thread::Builder::new()
///     .name("named thread".into())
///     .spawn(|| {
///         let handle = thread::current();
///         assert_eq!(handle.name(), Some("named thread"));
///     })
///     .unwrap();
///
/// handler.join().unwrap();
/// ```
#[must_use]
#[cfg(fw_has_thread_local)]
pub fn current() -> Thread {
    let current = CURRENT.get();
    if current > DESTROYED {
        unsafe {
            let current = ManuallyDrop::new(Thread::from_raw(current));
            (*current).clone()
        }
    } else {
        init_current(current)
    }
}

#[cold]
#[cfg(fw_has_thread_local)]
fn init_current(current: *mut ()) -> Thread {
    if current == NONE {
        CURRENT.set(BUSY);
        // If the thread ID was initialized already, use it.
        let id = id::get_or_init();
        let thread = Thread::new_current(id);

        // Make sure that `crate::rt::thread_cleanup` will be run, which will
        // call `drop_current`.
        crate::sys::thread_local::guard::enable();
        CURRENT.set(thread.clone().into_raw().cast_mut());
        thread
    } else if current == BUSY {
        // BUSY exists solely for this check, but as it is in the slow path, the
        // extra TLS write above shouldn't matter. The alternative is nearly always
        // a stack overflow.
        //
        // If we reach this point it means our initialization routine ended up
        // calling current() either directly, or indirectly through the global
        // allocator, which is a bug either way as we may not call the global
        // allocator in current().
        crate::rtabort!(
            "init_current() was re-entrant, which indicates a bug in the Rust threading \
             implementation"
        )
    } else {
        debug_assert_eq!(current, DESTROYED);
        panic!(
            "use of std::thread::current() is not possible after the thread's local data has been \
             destroyed"
        )
    }
}

/// This should be run in [`crate::rt::thread_cleanup`] to reset the thread
/// handle.
#[cfg(fw_has_thread_local)]
pub(crate) fn drop_current() {
    let current = CURRENT.get();
    if current > DESTROYED {
        unsafe {
            CURRENT.set(DESTROYED);
            drop(Thread::from_raw(current));
        }
    }
}

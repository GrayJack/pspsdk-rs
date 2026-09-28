use core::ptr;

use crate::sys::thread_local::key::{set, LazyKey};

/// On platforms with key-based TLS, the system runs the destructors for us.
/// We still have to make sure that [`crate::rt::thread_cleanup`] is called,
/// however. This is done by deferring the execution of a TLS destructor to
/// the next round of destruction inside the TLS destructors.
pub fn enable() {
    const DEFER: *mut u8 = ptr::without_provenance_mut(1);
    const RUN: *mut u8 = ptr::without_provenance_mut(2);

    static CLEANUP: LazyKey = LazyKey::new(Some(run));

    unsafe { set(CLEANUP.force(), DEFER) }

    unsafe extern "C" fn run(state: *mut u8) {
        if state == DEFER {
            // Make sure that this function is run again in the next round of
            // TLS destruction. If there is no further round, there will be leaks,
            // but that's okay, `thread_cleanup` is not guaranteed to be called.
            unsafe { set(CLEANUP.force(), RUN) }
        } else {
            debug_assert_eq!(state, RUN);
            // If the state is still RUN in the next round of TLS destruction,
            // it means that no other TLS destructors defined by this runtime
            // have been run, as they would have set the state to DEFER.
            crate::rt::thread_cleanup();
        }
    }
}

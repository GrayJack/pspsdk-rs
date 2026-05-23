use core::fmt;


pub mod nonpoison;
pub mod poison;

mod traits;

pub use traits::{RawMutex, RawMutexTimed, RawRwLock};

pub(crate) enum ExclusiveState {
    Incomplete,
    Poisoned,
    Complete,
}

/// State yielded to [`Once::call_once_force()`]’s closure parameter. The state
/// can be used to query the poison status of the [`Once`].
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
    /// use std::{sync::Once, thread};
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
    /// use oxilibc::platform::sync::poison::Once;
    ///
    /// static INIT: Once = Once::new();
    ///
    /// INIT.call_once_force(|state| {
    ///     assert!(!state.is_poisoned());
    /// });
    #[inline]
    pub fn is_poisoned(&self) -> bool {
        self.inner.is_poisoned()
    }

    /// Poison the associated [`Once`] without explicitly panicking.
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

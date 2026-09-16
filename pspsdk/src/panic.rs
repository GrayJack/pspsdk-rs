use core::{any::Any, fmt};

use alloc::{boxed::Box, string::String};

use crate::panicking;

pub use core::panic::*;


/// A struct providing information about a panic.
///
/// `PanicHookInfo` structure is passed to a panic hook set by the [`set_hook`] function.
///
/// # Examples
///
/// ```should_panic
/// use pspsdk::{panic, println};
///
/// panic::set_hook(Box::new(|panic_info| {
///     println!("panic occurred: {panic_info}");
/// }));
///
/// panic!("critical system failure");
/// ```
///
/// [`set_hook`]: ../../std/panic/fn.set_hook.html
#[derive(Debug)]
pub struct PanicHookInfo<'a> {
    payload: &'a (dyn Any + Send),
    location: &'static Location<'static>,
    can_unwind: bool,
    force_no_backtrace: bool,
}

impl<'a> PanicHookInfo<'a> {
    #[inline]
    pub(crate) fn new(
        location: &'static Location<'static>, payload: &'a (dyn Any + Send), can_unwind: bool,
        force_no_backtrace: bool,
    ) -> Self {
        PanicHookInfo {
            payload,
            location,
            can_unwind,
            force_no_backtrace,
        }
    }

    /// Returns the payload associated with the panic.
    ///
    /// This will commonly, but not always, be a `&'static str` or [`String`].
    /// If you only care about such payloads, use [`payload_as_str`] instead.
    ///
    /// A invocation of the `panic!()` macro in Rust 2021 or later will always result in a
    /// panic payload of type `&'static str` or `String`.
    ///
    /// Only an invocation of [`panic_any`]
    /// (or, in Rust 2018 and earlier, `panic!(x)` where `x` is something other than a string)
    /// can result in a panic payload other than a `&'static str` or `String`.
    ///
    /// [`String`]: ../../std/string/struct.String.html
    /// [`payload_as_str`]: PanicHookInfo::payload_as_str
    ///
    /// # Examples
    ///
    /// ```should_panic
    /// use pspsdk::{panic, println};
    ///
    /// panic::set_hook(Box::new(|panic_info| {
    ///     if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
    ///         println!("panic occurred: {s:?}");
    ///     } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
    ///         println!("panic occurred: {s:?}");
    ///     } else {
    ///         println!("panic occurred");
    ///     }
    /// }));
    ///
    /// panic!("Normal panic");
    /// ```
    #[must_use]
    #[inline]
    pub fn payload(&self) -> &(dyn Any + Send) {
        self.payload
    }

    /// Returns the payload associated with the panic, if it is a string.
    ///
    /// This returns the payload if it is of type `&'static str` or `String`.
    ///
    /// A invocation of the `panic!()` macro in Rust 2021 or later will always result in a
    /// panic payload where `payload_as_str` returns `Some`.
    ///
    /// Only an invocation of [`panic_any`]
    /// (or, in Rust 2018 and earlier, `panic!(x)` where `x` is something other than a string)
    /// can result in a panic payload where `payload_as_str` returns `None`.
    ///
    /// # Example
    ///
    /// ```should_panic
    /// pspsdk::panic::set_hook(Box::new(|panic_info| {
    ///     if let Some(s) = panic_info.payload_as_str() {
    ///         pspsdk::println!("panic occurred: {s:?}");
    ///     } else {
    ///         pspsdk::println!("panic occurred");
    ///     }
    /// }));
    ///
    /// panic!("Normal panic");
    /// ```
    #[must_use]
    #[inline]
    pub fn payload_as_str(&self) -> Option<&str> {
        if let Some(s) = self.payload.downcast_ref::<&str>() {
            Some(s)
        } else if let Some(s) = self.payload.downcast_ref::<String>() {
            Some(s)
        } else {
            None
        }
    }

    /// Returns information about the location from which the panic originated,
    /// if available.
    ///
    /// This method will currently always return [`Some`], but this may change
    /// in future versions.
    ///
    /// # Examples
    ///
    /// ```should_panic
    /// use pspsdk::{panic, println};
    ///
    /// panic::set_hook(Box::new(|panic_info| {
    ///     if let Some(location) = panic_info.location() {
    ///         println!("panic occurred in file '{}' at line {}", location.file(), location.line(),);
    ///     } else {
    ///         println!("panic occurred but can't get location information...");
    ///     }
    /// }));
    ///
    /// panic!("Normal panic");
    /// ```
    #[must_use]
    #[inline]
    pub fn location(&self) -> Option<&'static Location<'static>> {
        // NOTE: If this is changed to sometimes return None,
        // deal with that case in pspsdk::panicking::default_hook and core::panicking::panic_fmt.
        Some(self.location)
    }

    /// Returns whether the panic handler is allowed to unwind the stack from
    /// the point where the panic occurred.
    ///
    /// This is true for most kinds of panics with the exception of panics
    /// caused by trying to unwind out of a `Drop` implementation or a function
    /// whose ABI does not support unwinding.
    ///
    /// It is safe for a panic handler to unwind even when this function returns
    /// false, however this will simply cause the panic handler to be called
    /// again.
    #[must_use]
    #[inline]
    pub fn can_unwind(&self) -> bool {
        self.can_unwind
    }

    #[doc(hidden)]
    #[inline]
    pub fn force_no_backtrace(&self) -> bool {
        self.force_no_backtrace
    }
}

impl fmt::Display for PanicHookInfo<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("panicked at ")?;
        self.location.fmt(formatter)?;
        if let Some(payload) = self.payload_as_str() {
            formatter.write_str(":\n")?;
            formatter.write_str(payload)?;
        }
        Ok(())
    }
}

/// Invokes a closure, capturing the cause of an unwinding panic if one occurs.
///
/// This function will return `Ok` with the closure's result if the closure does
/// not panic, and will return `Err(cause)` if the closure panics. The `cause`
/// returned is the object with which panic was originally invoked.
///
/// Rust functions that are expected to be called from foreign code that does
/// not support unwinding (such as C compiled with `-fno-exceptions`) should be
/// defined using `extern "C"`, which ensures that if the Rust code panics, it
/// is automatically caught and the process is aborted. If this is the desired
/// behavior, it is not necessary to use `catch_unwind` explicitly. This
/// function should instead be used when more graceful error-handling is needed.
///
/// It is **not** recommended to use this function for a general try/catch
/// mechanism. The [`Result`] type is more appropriate to use for functions that
/// can fail on a regular basis. Additionally, this function is not guaranteed
/// to catch all panics, see the "Notes" section below.
///
/// The closure provided is required to adhere to the [`UnwindSafe`] trait to
/// ensure that all captured variables are safe to cross this boundary. The
/// purpose of this bound is to encode the concept of [exception safety][rfc] in
/// the type system. Most usage of this function should not need to worry about
/// this bound as programs are naturally unwind safe without `unsafe` code. If
/// it becomes a problem the [`AssertUnwindSafe`] wrapper struct can be used to
/// quickly assert that the usage here is indeed unwind safe.
///
/// [rfc]: https://rust-lang.github.io/rfcs/1236-stabilize-catch-panic.html
///
/// # Notes
///
/// This function **might not catch all Rust panics**. A Rust panic is not
/// always implemented via unwinding, but can be implemented by aborting the
/// process as well. This function *only* catches unwinding panics, not those
/// that abort the process.
///
/// If a custom panic hook has been set, it will be invoked before the panic is
/// caught, before unwinding.
///
/// Although unwinding into Rust code with a foreign exception (e.g. an
/// exception thrown from C++ code, or a `panic!` in Rust code compiled or
/// linked with a different runtime) via an appropriate ABI (e.g. `"C-unwind"`)
/// is permitted, catching such an exception using this function will have one
/// of two behaviors, and it is unspecified which will occur:
///
/// * The process aborts, after executing all destructors of `f` and the functions it called.
/// * The function returns a `Result::Err` containing an opaque type.
///
/// Finally, be **careful in how you drop the result of this function**. If it
/// is `Err`, it contains the panic payload, and dropping that may in turn
/// panic!
///
/// # Examples
///
/// ```
/// use pspsdk::panic;
///
/// let result = panic::catch_unwind(|| {
///     println!("hello!");
/// });
/// assert!(result.is_ok());
///
/// let result = panic::catch_unwind(|| {
///     panic!("oh no!");
/// });
/// assert!(result.is_err());
/// ```
pub fn catch_unwind<F: FnOnce() -> R + UnwindSafe, R>(f: F) -> Result<R, Box<dyn Any + Send>> {
    unsafe { panicking::catch_unwind(f) }
}

/// Triggers a panic without invoking the panic hook.
///
/// This is designed to be used in conjunction with [`catch_unwind`] to, for
/// example, carry a panic across a layer of C code.
///
/// # Notes
///
/// Note that panics in Rust are not always implemented via unwinding, but they
/// may be implemented by aborting the process. If this function is called when
/// panics are implemented this way then this function will abort the process,
/// not trigger an unwind.
///
/// # Examples
///
/// ```should_panic
/// use pspsdk::panic;
///
/// let result = panic::catch_unwind(|| {
///     if 1 != 2 {
///         panic!("oh no!");
///     }
/// });
///
/// if let Err(err) = result {
///     panic::resume_unwind(err);
/// }
/// ```
pub fn resume_unwind(payload: Box<dyn Any + Send>) -> ! {
    panicking::resume_unwind(payload)
}

/// Makes all future panics abort directly without running the panic hook or unwinding.
///
/// There is no way to undo this; the effect lasts until the process exits or
/// execs (or the equivalent).
///
/// # Use after fork
///
/// This function is particularly useful for calling after `libc::fork`.  After `fork`, in a
/// multithreaded program it is (on many platforms) not safe to call the allocator.  It is also
/// generally highly undesirable for an unwind to unwind past the `fork`, because that results in
/// the unwind propagating to code that was only ever expecting to run in the parent.
///
/// `panic::always_abort()` helps avoid both of these.  It directly avoids any further unwinding,
/// and if there is a panic, the abort will occur without allocating provided that the arguments to
/// panic can be formatted without allocating.
///
/// Examples
///
/// ```no_run
/// use pspsdk::panic;
///
/// panic::always_abort();
///
/// let _ = panic::catch_unwind(|| {
///     panic!("inside the catch");
/// });
///
/// // We will have aborted already, due to the panic.
/// unreachable!();
/// ```
pub fn always_abort() {
    crate::panicking::panic_count::set_always_abort();
}

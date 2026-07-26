//! A module for working with processes.
//!
//! Provides [`abort`] and [`exit`] for terminating the current process.

use crate::sys::{self, SceResult, SceResultOk};

/// This type represents the status code the current process can return
/// to its parent under normal termination.
///
/// `ExitCode` is intended to be consumed only by the standard library (via
/// [`Termination::report()`]). For forwards compatibility with potentially
/// unusual targets, this type currently does not provide `Eq`, `Hash`, or
/// access to the raw value. This type does provide `PartialEq` for
/// comparison, but note that there may potentially be multiple failure
/// codes, some of which will _not_ compare equal to `ExitCode::FAILURE`.
/// The standard library provides the canonical `SUCCESS` and `FAILURE`
/// exit codes as well as `From<u8> for ExitCode` for constructing other
/// arbitrary exit codes.
///
/// # Portability
///
/// Numeric values used in this type don't have portable meanings, and
/// different platforms may mask different amounts of them.
///
/// For the platform's canonical successful and unsuccessful codes, see
/// the [`SUCCESS`] and [`FAILURE`] associated items.
///
/// [`SUCCESS`]: ExitCode::SUCCESS
/// [`FAILURE`]: ExitCode::FAILURE
// # Differences from `ExitStatus`
//
// `ExitCode` is intended for terminating the currently running process, via
// the `Termination` trait, in contrast to [`ExitStatus`], which represents the
// termination of a child process. These APIs are separate due to platform
// compatibility differences and their expected usage; it is not generally
// possible to exactly reproduce an `ExitStatus` from a child for the current
// process after the fact.
/// # Examples
///
/// `ExitCode` can be returned from the `main` function of a crate, as it implements
/// [`Termination`]:
///
/// ```
/// use pspsdk::process::ExitCode;
/// # fn check_foo() -> bool { true }
///
/// fn main() -> ExitCode {
///     if !check_foo() {
///         return ExitCode::from(42);
///     }
///
///     ExitCode::SUCCESS
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExitCode(i32);

impl ExitCode {
    /// The canonical `ExitCode` for unsuccessful termination on this platform.
    ///
    /// If you're only returning this and `SUCCESS` from `main`, consider
    /// instead returning `Err(_)` and `Ok(())` respectively, which will
    /// return the same codes (but will also `eprintln!` the error).
    pub const FAILURE: ExitCode = ExitCode(-1);
    /// The canonical `ExitCode` for successful termination on this platform.
    ///
    /// Note that a `()`-returning `main` implicitly results in a successful
    /// termination, so there's no need to return this from `main` unless
    /// you're also returning other possible codes.
    pub const SUCCESS: ExitCode = ExitCode(0);

    /// Creates a custom exit code.
    #[inline]
    pub const fn new(raw_exit_code: i32) -> Self {
        Self(raw_exit_code)
    }

    #[inline]
    #[doc(hidden)]
    pub const fn to_i32(self) -> i32 {
        self.0
    }
}

/// A trait for implementing arbitrary return types in the `main` function.
///
/// The C-main function only supports returning integers.
/// So, every type implementing the `Termination` trait has to be converted
/// to an integer.
///
/// The default implementations are returning `libc::EXIT_SUCCESS` to indicate
/// a successful execution. In case of a failure, `libc::EXIT_FAILURE` is returned.
///
/// Because different runtimes have different specifications on the return value
/// of the `main` function, this trait is likely to be available only on
/// standard library's runtime for convenience. Other runtimes are not required
/// to provide similar functionality.
pub trait Termination {
    /// Is called to get the representation of the value as status code.
    /// This status code is returned to the operating system.
    fn report(self) -> ExitCode;
}

impl Termination for () {
    #[inline]
    fn report(self) -> ExitCode {
        ExitCode::SUCCESS
    }
}

impl Termination for ! {
    fn report(self) -> ExitCode {
        self
    }
}

impl Termination for core::convert::Infallible {
    fn report(self) -> ExitCode {
        match self {}
    }
}

impl Termination for ExitCode {
    #[inline]
    fn report(self) -> ExitCode {
        self
    }
}

impl<T: Termination, E: core::fmt::Debug> Termination for Result<T, E> {
    fn report(self) -> ExitCode {
        match self {
            Ok(val) => val.report(),
            Err(err) => {
                crate::io::printing::attempt_print_to_stderr(format_args!("Error: {err:?}\n"));
                ExitCode::FAILURE
            },
        }
    }
}

impl<T: Termination + SceResultOk> Termination for SceResult<T> {
    fn report(self) -> ExitCode {
        match self.into_result() {
            Ok(val) => val.report(),
            Err(err) => {
                crate::io::printing::attempt_print_to_stderr(format_args!("Error: {err:?}\n"));
                ExitCode::new(err.to_inner().cast_signed())
            },
        }
    }
}

/// An guard of suspended interrupts that resumes when dropped (falls out of scope).
///
/// This structure is created by the [`suspend_interrupts`] function.
#[derive(Debug)]
#[must_use = "if unused the interrupts will immediately resume"]
pub struct SuspendInterruptsGuard {
    previous: u32,
}

impl Drop for SuspendInterruptsGuard {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: `SuspendInterruptsGuard` only exists while interrupts were suspended
        unsafe {
            sys::resume_interrupts(self.previous);
        }
    }
}

/// Suspends interrupts, returning a guard that will resume interrupts on drop.
///
/// # Examples
///
/// ```no_run
/// #![no_std]
/// #![no_main]
///
/// use pspsdk::{io, process};
///
/// fn psp_main() -> io::Result<()> {
///     pspsdk::enable_home_button();
///
///     // stuff...
///     {
///         let intr_guard = process::suspend_interrupts();
///         // Do things with interrupt suspended
///     } // interrupts resumed
/// }
/// ```
#[inline]
pub fn suspend_interrupts() -> SuspendInterruptsGuard {
    let previous = unsafe { sys::suspend_interrupts() };
    SuspendInterruptsGuard { previous }
}

/// Executes a procedure with the interrupts suspended.
///
/// Returns the result of the procedure.
///
/// # Examples
///
/// ```no_run
/// #![no_std]
/// #![no_main]
///
/// use pspsdk::process;
///
/// # fn psp_main() -> pspsdk::io::Result<()> {
/// #    pspsdk::enable_home_button();
///
/// let mut ans = 0;
/// let res = process::with_suspended_interrupts(|| {
///     ans = 42;
///     true
/// });
///
/// assert!(res);
/// assert_eq!(ans, 42);
/// # }
/// ```
pub fn with_suspended_interrupts<F, R>(mut f: F) -> R
where
    F: FnMut() -> R,
{
    let guard = suspend_interrupts();
    let res = f();
    drop(guard);
    res
}

/// Returns `true` if the the interrupt is enabled, `false` otherwise.
///
/// # Examples
///
/// ```no_run
/// #![no_std]
/// #![no_main]
///
/// use pspsdk::process;
///
/// # fn psp_main() -> pspsdk::io::Result<()> {
/// #    pspsdk::enable_home_button();
///
/// if process::is_interrupt_enabled() { /* Do something that requires interrupt enabled */ }
///
/// # }
/// ```
#[inline]
pub fn is_interrupt_enabled() -> bool {
    sys::is_interrupt_enabled()
}

/// Terminates the current process with the specified exit code.
///
/// This function will never return and will immediately terminate the current
/// process. The exit code is passed through to the underlying OS and will be
/// available for consumption by another process.
///
/// Note that because this function never returns, and that it terminates the
/// process, no destructors on the current stack or any other thread's stack
/// will be run. If a clean shutdown is needed it is recommended to only call
/// this function at a known point where there are no more destructors left
/// to run; or, preferably, simply return a type implementing [`Termination`]
/// (such as [`ExitCode`] or `Result`) from the `main` function and avoid this
/// function altogether:
///
/// ```
/// # use std::io::Error as MyError;
/// fn main() -> Result<(), MyError> {
///     // ...
///     Ok(())
/// }
/// ```
///
/// In its current implementation, this function will execute exit handlers registered with `atexit`
/// as well as other platform-specific exit handlers (e.g. `fini` sections of ELF shared objects).
/// This means that Rust requires that all exit handlers are safe to execute at any time. In
/// particular, if an exit handler cleans up some state that might be concurrently accessed by other
/// threads, it is required that the exit handler performs suitable synchronization with those
/// threads. (The alternative to this requirement would be to not run exit handlers at all, which is
/// considered undesirable. Note that returning from `main` also calls `exit`, so making `exit` an
/// unsafe operation is not an option.)
///
/// ## Platform-specific behavior
///
/// **Unix**: On Unix-like platforms, it is unlikely that all 32 bits of `exit`
/// will be visible to a parent process inspecting the exit code. On most
/// Unix-like platforms, only the eight least-significant bits are considered.
///
/// For example, the exit code for this example will be `0` on Linux, but `256`
/// on Windows:
///
/// ```no_run
/// use std::process;
///
/// process::exit(0x0100);
/// ```
///
/// ### Safe interop with C code
///
/// On Unix, this function is currently implemented using the `exit` C function [`exit`][C-exit]. As
/// of C23, the C standard does not permit multiple threads to call `exit` concurrently. Rust
/// mitigates this with a lock, but if C code calls `exit`, that can still cause undefined behavior.
/// Note that returning from `main` is equivalent to calling `exit`.
///
/// Therefore, it is undefined behavior to have two concurrent threads perform the following
/// without synchronization:
/// - One thread calls Rust's `exit` function or returns from Rust's `main` function
/// - Another thread calls the C function `exit` or `quick_exit`, or returns from C's `main`
///   function
///
/// Note that if a binary contains multiple copies of the Rust runtime (e.g., when combining
/// multiple `cdylib` or `staticlib`), they each have their own separate lock, so from the
/// perspective of code running in one of the Rust runtimes, the "outside" Rust code is basically C
/// code, and concurrent `exit` again causes undefined behavior.
///
/// Individual C implementations might provide more guarantees than the standard and permit
/// concurrent calls to `exit`; consult the documentation of your C implementation for details.
///
/// For some of the on-going discussion to make `exit` thread-safe in C, see:
/// - [Rust issue #126600](https://github.com/rust-lang/rust/issues/126600)
/// - [Austin Group Bugzilla (for POSIX)](https://austingroupbugs.net/view.php?id=1845)
/// - [GNU C library Bugzilla](https://sourceware.org/bugzilla/show_bug.cgi?id=31997)
///
/// [C-exit]: https://en.cppreference.com/w/c/program/exit
pub fn exit(code: i32) -> ! {
    crate::rt::cleanup();

    let main_thread_id = crate::rt::main_thread_id();

    if let Some(main_thread_id) = main_thread_id {
        let thread_id = sys::thread::sceKernelGetThreadId().into_result();
        if let Ok(curr_thread_id) = thread_id
            && curr_thread_id == main_thread_id
        {
            exit_main(code)
        }
    }

    loop {
        if crate::process::is_interrupt_enabled() {
            let _ = sys::thread::sceKernelExitDeleteThread(code.cast_unsigned());
        }
    }
}

/// Terminates the process in an abnormal fashion.
///
/// The function will never return and will immediately terminate the current
/// process in a platform specific "abnormal" manner. As a consequence,
/// no destructors on the current stack or any other thread's stack
/// will be run, Rust IO buffers (eg, from `BufWriter`) will not be flushed,
/// and C stdio buffers will (on most platforms) not be flushed.
///
/// This is in contrast to the default behavior of [`panic!`] which unwinds
/// the current thread's stack and calls all destructors.
/// When `panic="abort"` is set, either as an argument to `rustc` or in a
/// crate's Cargo.toml, [`panic!`] and `abort` are similar. However,
/// [`panic!`] will still call the panic hook while `abort` will not.
///
/// If a clean shutdown is needed it is recommended to only call
/// this function at a known point where there are no more destructors left
/// to run.
///
/// The process's termination will be similar to that from the C `abort()`
/// function.  On Unix, the process will terminate with signal `SIGABRT`, which
/// typically means that the shell prints "Aborted".
///
/// # Examples
///
/// ```no_run
/// #![no_std]
/// #![no_main]
///
/// use pspsdk::process;
///
/// fn psp_main() {
///     println!("aborting");
///
///     process::abort();
///
///     // execution never gets here
/// }
/// ```
///
/// The `abort` function terminates the process, so the destructor will not
/// get run on the example below:
///
/// ```no_run
/// #![no_std]
/// #![no_main]
///
/// use pspsdk::process;
///
/// struct HasDrop;
///
/// impl Drop for HasDrop {
///     fn drop(&mut self) {
///         pspsdk::println!("This will never be printed!");
///     }
/// }
///
/// fn psp_main() {
///     let _x = HasDrop;
///     process::abort();
///     // the destructor implemented for HasDrop will never get run
/// }
/// ```
#[cold]
#[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
#[allow(clippy::never_loop)]
pub fn abort() -> ! {
    if cfg!(debug_assertions) {
        crate::println!("Abort called");
    }

    loop {
        cfg_select! {
            prx => {
                if crate::process::is_interrupt_enabled() {
                    let _ = unsafe { crate::sys::module::sceKernelSelfStopUnloadModule(0xDEADCAFE, 0, core::ptr::null_mut()) };
                }
            },
            all(pbp, feature = "kernel") => {
                if crate::process::is_interrupt_enabled() {
                    let _ = crate::sys::loadexec::sceKernelExitVSHVSH(None);
                }
            }
            all(pbp, not(feature = "kernel")) => {
                if crate::process::is_interrupt_enabled() {
                    let _ = crate::sys::loadexec::sceKernelExitGameWithStatus(0xDEADCAFE);
                }
            }
            _ => core::intrinsics::abort(),
        }
    }
}

#[allow(clippy::never_loop)]
pub(crate) fn exit_main(status: i32) -> ! {
    loop {
        cfg_select! {
            prx => {
                if crate::process::is_interrupt_enabled() {
                    let _ = unsafe { crate::sys::module::sceKernelSelfStopUnloadModule(status.cast_unsigned(), 0, core::ptr::null_mut()) };
                }
            },
            all(pbp, feature = "kernel") => {
                if crate::process::is_interrupt_enabled() {
                    let _ = crate::sys::loadexec::sceKernelExitVSHVSH(None);
                }
            }
            all(pbp, not(feature = "kernel")) => {
                if crate::process::is_interrupt_enabled() {
                    let _ = crate::sys::loadexec::sceKernelExitGameWithStatus(status.cast_unsigned());
                }
            }
            _ => {
                if crate::process::is_interrupt_enabled() {
                    let _ = sys::thread::sceKernelExitDeleteThread(status.cast_unsigned());
                }
            },
        }
    }
}

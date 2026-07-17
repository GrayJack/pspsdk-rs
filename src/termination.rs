//! Modified `std::process::Termination` for this PSP SDK.

use crate::sys::{SceResult, SceResultOk};

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
///
/// # Differences from `ExitStatus`
///
/// `ExitCode` is intended for terminating the currently running process, via
/// the `Termination` trait, in contrast to [`ExitStatus`], which represents the
/// termination of a child process. These APIs are separate due to platform
/// compatibility differences and their expected usage; it is not generally
/// possible to exactly reproduce an `ExitStatus` from a child for the current
/// process after the fact.
///
/// # Examples
///
/// `ExitCode` can be returned from the `main` function of a crate, as it implements
/// [`Termination`]:
///
/// ```
/// use std::process::ExitCode;
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

    #[inline]
    #[doc(hidden)]
    pub fn to_i32(self) -> i32 {
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

// impl Termination for ! {
//     fn report(self) -> ExitCode {
//         self
//     }
// }

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
                ExitCode::FAILURE
            },
        }
    }
}

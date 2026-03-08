use core::marker::PhantomData;

use bitflag_attr::bitflag;

#[doc(hidden)]
#[cfg(target_os = "psp")]
pub mod macro_helpers;


mod error;
pub use error::{SceError, SceErrorFacility};

pub mod atrac;
pub mod library;

#[cfg(target_os = "psp")]
pub type SceSize = usize;
#[cfg(not(target_os = "psp"))]
pub type SceSize = u32;

#[cfg(target_os = "psp")]
pub type SceIsize = isize;
#[cfg(not(target_os = "psp"))]
pub type SceIsize = i32;

/// Identification number for several kernel objects.
#[rustc_layout_scalar_valid_range_start(0)]
#[rustc_layout_scalar_valid_range_end(0x7FFFFFFF)]
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceUid(u32);

impl SceUid {
    /// Create a new SceUid structure from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible SceUid
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn new(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::new_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new UID structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn new_unchecked(raw: u32) -> Self {
        // SAFETY: Caller promised that `val` is within the valid range.
        unsafe { Self(raw) }
    }

    #[inline]
    pub const fn as_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for SceUid {}

/// A type that represents the return value of many PSP OS APIs.
///
/// If its value is in the range of [`SceError`] ( 0x80000001..=0xFFFFFFFF), then it is an error
/// result, and a success value otherwise.
///
/// # Cloning behavior
///
/// Cloning this type is cheat, the reason it is not [`Copy`] is to incentivize the handling of the
/// error values.
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[must_use = "this `SceResult` may be an error value, which should be handled"]
pub struct SceResult<T>(u32, PhantomData<T>);

impl<T> SceResult<T> {
    /// Create a new SceResult.
    pub const fn new(raw: u32) -> Self {
        SceResult(raw, PhantomData)
    }

    /// Returns `true` if the result is a Ok value.
    #[inline]
    pub const fn is_ok(&self) -> bool {
        matches!(self.as_inner(), 0..=0x7FFFFFFF)
    }

    /// Returns `true` if the result is a error value.
    #[inline]
    pub const fn is_err(&self) -> bool {
        matches!(self.as_inner(), 0x80000001..=0xFFFFFFFF)
    }

    /// Get the internal value of the result.
    #[inline]
    pub(crate) const fn as_inner(&self) -> u32 {
        self.0
    }
}

impl<T: SceResultOk> SceResult<T> {
    /// Turn the SceResult into a [`Result`] type.
    pub fn into_result(self) -> Result<T, SceError> {
        match self.as_inner() {
            0..=0x7FFFFFFF => unsafe { T::handle_ok_value(self.as_inner()) },
            0x80000001..=0xFFFFFFFF => Err(unsafe { SceError::new_unchecked(self.as_inner()) }),
            0x80000000 => Err(SceError::INVALID_VALUE),
        }
    }

    /// Converts from `SceResult` to [`Option<T>`].
    ///
    /// Converts `self` into an [`Option<T>`], consuming `self`,
    /// and discarding the error, if any.
    pub fn ok(self) -> Option<T> {
        self.into_result().ok()
    }

    /// Converts from `SceResult<T>` to [`Option<SceError>`].
    ///
    /// Converts `self` into an [`Option<SceError>`], consuming `self`,
    /// and discarding the success value, if any.
    pub fn err(self) -> Option<SceError> {
        self.into_result().err()
    }

    // Maps a `SceResult<T>` to `Result<U, E>` by applying a function to a
    /// contained [`Ok`] value, leaving an [`Err`] value untouched.
    ///
    /// This function can be used to compose the results of two functions.
    pub fn map<U, F>(self, op: F) -> Result<U, SceError>
    where
        F: FnOnce(T) -> U,
    {
        self.into_result().map(op)
    }

    /// Returns the provided default (if `Err`), or
    /// applies a function to the contained value (if `Ok`).
    ///
    /// Arguments passed to `map_or` are eagerly evaluated; if you are passing
    /// the result of a function call, it is recommended to use [`map_or_else`],
    /// which is lazily evaluated.
    ///
    /// [`map_or_else`]: SceResult::map_or_else
    pub fn map_or<U, F>(self, default: U, f: F) -> U
    where
        F: FnOnce(T) -> U,
    {
        self.into_result().map_or(default, f)
    }

    /// Maps a `SceResult<T>` to `U` by applying fallback function `default` to
    /// a contained `Err` value, or function `f` to a contained `Ok` value.
    ///
    /// This function can be used to unpack a successful result
    /// while handling an error.
    pub fn map_or_else<U, D, F>(self, default: D, f: F) -> U
    where
        D: FnOnce(SceError) -> U,
        F: FnOnce(T) -> U,
    {
        self.into_result().map_or_else(default, f)
    }

    /// Maps a `SceResult<T>` to a `U` by applying function `f` to the contained
    /// value if the result is `Ok`], otherwise if `Err`, returns the
    /// [default value] for the type `U`.
    ///
    /// [default value]: Default::default
    pub fn map_or_default<U, F>(self, f: F) -> U
    where
        F: FnOnce(T) -> U,
        U: Default,
    {
        match self.into_result() {
            Ok(t) => f(t),
            Err(_) => U::default(),
        }
    }

    /// Maps a `ScwResult<T>` to `Result<T, F>` by applying a function to a
    /// contained `Err` value, leaving an `Ok` value untouched.
    ///
    /// This function can be used to pass through a successful result while handling
    /// an error.
    pub fn map_err<F, O>(self, op: O) -> Result<T, F>
    where
        O: FnOnce(SceError) -> F,
    {
        self.into_result().map_err(op)
    }

    /// Calls a function with a reference to the contained value if `Ok` value.
    ///
    /// Returns the original result.
    pub fn inspect<F>(self, f: F) -> Self
    where
        F: FnOnce(&T),
    {
        if self.is_ok() {
            // SAFETY: We know it is a Ok value
            let res = unsafe { T::handle_ok_value(self.as_inner()) };
            if let Ok(inner) = res {
                f(&inner)
            }
        }

        self
    }

    /// Calls a function with a reference to the contained value if `Err` value.
    ///
    /// Returns the original result.
    pub fn inspect_err<F>(self, f: F) -> Self
    where
        F: FnOnce(SceError),
    {
        if self.is_err() {
            // SAFETY: we know it is an error
            f(unsafe { SceError::new_unchecked(self.as_inner()) })
        }

        self
    }
}

/// Trait of types that can be the ok result of [`SceResult`].
///
/// # Safety
///
/// For this trait to be correct, the implementation must:
/// - Be valid for the `0..=0x7FFFFFFF` range
/// - Have a max size of 4 bytes
/// - If a struct, be `repr(transparent)`
pub unsafe trait SceResultOk: Sized + crate::private::Sealed {
    /// Turn the Ok value range (`(0,0x7FFFFFFF]`) into a result
    ///
    /// The `ok_value` is always in the ok value range when used by [`SceResult`].
    ///
    /// # Safety
    /// [`SceResult`] methods that use it will ensure to pass only the valid range and the
    /// implementation is free to not check for values outside of that range.
    ///
    /// For callers outside of [`SceResult`], it must ensure to pass a valid `ok_value`.
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError>;
}

macro_rules! __result_ok_int {
    ($($ty:ty),+) => {
        $(
            unsafe impl SceResultOk for $ty {
                unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
                    <$ty>::try_from(ok_value).map_err(|_| SceError::INVALID_VALUE)
                }
            }
        )+
    };
}

__result_ok_int!(i8, u8, i16, u16, bool);


unsafe impl SceResultOk for i32 {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        debug_assert!(ok_value <= 0x7FFFFFFF);
        Ok(u32::cast_signed(ok_value))
    }
}
unsafe impl SceResultOk for u32 {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        debug_assert!(ok_value <= 0x7FFFFFFF);
        Ok(ok_value)
    }
}
#[cfg(target_pointer_width = "32")]
unsafe impl SceResultOk for isize {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        isize::try_from(ok_value).map_err(|_| SceError::INVALID_VALUE)
    }
}
#[cfg(target_pointer_width = "32")]
unsafe impl SceResultOk for usize {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        usize::try_from(ok_value).map_err(|_| SceError::INVALID_VALUE)
    }
}
unsafe impl SceResultOk for SceUid {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        debug_assert!(ok_value <= 0x7FFFFFFF);
        // SAFETY: SceUid is always on the ok value range
        Ok(unsafe { Self::new_unchecked(ok_value) })
    }
}

/// Resident/Stub library attributes.
///
/// Every library needs to have at least one of those attributes.
///
/// Resident libraries can have the members [`AutoExport`](SceLibFlags::AutoExport),
/// [`WeakExport`](SceLibFlags::WeakExport), [`NoLinkExport`](SceLibFlags::NoLinkExport),
/// [`SyscallExport`](SceLibFlags::SyscallExport) and [`IsSystemLib`](SceLibFlags::IsSystemLib).
///
/// Stub libraries can have [`NoSpecialFlags`](SceLibFlags::NoSpecialFlags) or
/// [`WeakImport`](SceLibFlags::WeakImport).
#[bitflag(u16)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum SceLibFlags {
    /// The library has no special attributes.
    NoSpecialFlags = 0x0,
    /// Automatically register the library to the system.
    AutoExport = 0x1,
    /// Indicates resident library can be overwritten.
    WeakExport = 0x2,
    /// Indicates resident library is NOT being linked.
    NoLinkExport = 0x4,
    /// Load module that references this library even if this library is not registered.
    WeakImport = 0x8,
    /// Indicates the use of the SYSCALL technique for linking.
    SyscallExport = 0x4000,
    /// The library is a system library (a mandatory library for all modules).
    IsSystemLib = 0x8000,
}

//! The low-level implementations, type definition, and system stubs for the PSP system.
use core::{marker::PhantomData, ops::ControlFlow};

use bitflag_attr::bitflag;

#[doc(hidden)]
#[cfg(target_os = "psp")]
pub mod macro_helpers;


mod error;
pub use error::{ErrorFacility, SceError};

pub mod audio;
pub mod ctrl;
pub mod display;
pub mod dma;
pub mod ge;
pub mod hprm;
pub mod io;
pub mod libc;
pub mod library;
pub mod loadexec;
pub mod mem;
pub mod module;
pub mod openpsid;
pub mod power;
pub mod suspend;
pub mod thread;
pub mod time;
pub mod usersystemlib;

#[cfg(feature = "non-stub-code")]
pub mod sync;

/// A `usize`-like with the guarantee to be the correct size on PSP.
pub type SceSize = cfg_select! {
    target_os = "psp" => usize,
    target_pointer_width = "32" => usize,
    _ => u32,
};

/// A `isize`-like with the guarantee to be the correct size on PSP.
pub type SceIsize = cfg_select! {
    target_os = "psp" => isize,
    target_pointer_width = "32" => isize,
    _ => i32,
};

/// Identification number for several kernel objects.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SceUid(pattern_type!(SceRawUid is 0..=0x7FFFFFFF));

/// Identification number for several kernel objects.
pub type SceRawUid = u32;

impl SceUid {
    /// Create a new SceUid structure from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible SceUid
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::from_raw_unchecked(raw) })
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
    pub const unsafe fn from_raw_unchecked(raw: u32) -> Self {
        // SAFETY: Caller promised that `val` is within the valid range.
        unsafe { core::mem::transmute(raw) }
    }

    #[inline]
    pub const fn to_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

crate::impl_ranged_ty!(SceUid);

impl crate::private::Sealed for SceUid {}

impl Default for SceUid {
    fn default() -> Self {
        unsafe { Self::from_raw_unchecked(0) }
    }
}

impl core::fmt::Debug for SceUid {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_tuple("SceUid").field(&self.to_inner()).finish()
    }
}

/// A type that represents the return value of many PSP OS APIs.
///
/// If its value is in the range of [`SceError`] ( 0x80000001..=0xFFFFFFFF), then it is an error
/// result, and a success value otherwise.
///
/// # Cloning behavior
///
/// Cloning this type is cheap, the reason it is not [`Copy`] is to incentivize the handling of the
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
            0..=0x7FFFFFFF => unsafe {
                T::handle_ok_value(self.as_inner()).ok_or(SceError::INVALID_VALUE)
            },
            0x80000001..=0xFFFFFFFF => {
                Err(unsafe { SceError::from_raw_unchecked(self.as_inner()) })
            },
            0x80000000 => Err(SceError::INVALID_VALUE),
        }
    }

    /// Converts from `SceResult` to [`Option<T>`].
    ///
    /// Converts `self` into an [`Option<T>`], consuming `self`,
    /// and discarding the error, if any.
    pub fn ok(self) -> Option<T> {
        match self.as_inner() {
            0..=0x7FFFFFFF => unsafe { T::handle_ok_value(self.as_inner()) },
            _ => None,
        }
    }

    /// Converts from `SceResult<T>` to [`Option<SceError>`].
    ///
    /// Converts `self` into an [`Option<SceError>`], consuming `self`,
    /// and discarding the success value, if any.
    pub fn err(self) -> Option<SceError> {
        match self.as_inner() {
            0..=0x7FFFFFFF => None,
            0x80000001..=0xFFFFFFFF => {
                Some(unsafe { SceError::from_raw_unchecked(self.as_inner()) })
            },
            0x80000000 => Some(SceError::INVALID_VALUE),
        }
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
            if let Some(inner) = res {
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
            f(unsafe { SceError::from_raw_unchecked(self.as_inner()) })
        }

        self
    }
}

/// A type that represents the return value of some PSP OS APIs.
///
/// If its value is in the range of [`SceError`] ( 0xFFFFFFFF_80000001..=0xFFFFFFFF_FFFFFFFF), then
/// it is an error result, and a success value otherwise.
///
/// # Cloning behavior
///
/// Cloning this type is cheap, the reason it is not [`Copy`] is to incentivize the handling of the
/// error values.
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[must_use = "this `SceResult` may be an error value, which should be handled"]
pub struct SceResult64<T>(u64, PhantomData<T>);

impl<T> SceResult64<T> {
    /// Create a new SceResult.
    pub const fn new(raw: u64) -> Self {
        SceResult64(raw, PhantomData)
    }

    /// Returns `true` if the result is a Ok value.
    #[inline]
    pub const fn is_ok(&self) -> bool {
        matches!(self.as_inner(), 0..=0xFFFFFFFF_7FFFFFFF)
    }

    /// Returns `true` if the result is a error value.
    #[inline]
    pub const fn is_err(&self) -> bool {
        matches!(self.as_inner(), 0xFFFFFFFF_80000001..=0xFFFFFFFF_FFFFFFFF)
    }

    /// Get the internal value of the result.
    #[inline]
    pub(crate) const fn as_inner(&self) -> u64 {
        self.0
    }
}

impl<T: SceResultOk> SceResult64<T> {
    /// Turn the SceResult64 into a [`Result`] type.
    pub fn into_result(self) -> Result<T, SceError> {
        match self.as_inner() {
            0..=0xFFFFFFFF_7FFFFFFF => unsafe {
                T::handle_ok_value64(self.as_inner()).ok_or(SceError::INVALID_VALUE)
            },
            0xFFFFFFFF_80000001..=0xFFFFFFFF_FFFFFFFF => {
                // Only lower bits
                let err = (self.as_inner() & 0xFFFFFFFF_00000000) as u32;
                Err(unsafe { SceError::from_raw_unchecked(err) })
            },
            0xFFFFFFFF_80000000 => Err(SceError::INVALID_VALUE),
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
            let res = unsafe { T::handle_ok_value64(self.as_inner()) };
            if let Some(inner) = res {
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
            // Only lower bits
            let err = (self.as_inner() & 0xFFFFFFFF_00000000) as u32;
            // SAFETY: we know it is an error
            let err = unsafe { SceError::from_raw_unchecked(err) };
            f(err)
        }

        self
    }
}

impl SceResult<()> {
    /// Everything ok.
    pub const OK: Self = SceResult::new(0);
}

/// Trait of types that can be the ok result of [`SceResult`] or [`SceResult64`].
///
/// # Safety
///
/// For this trait to be correct, the implementation must:
/// - Be valid for the `0..=0x7FFFFFFF` range
/// - Have a max size of 4 bytes
/// - If a struct, be `repr(transparent)`
pub unsafe trait SceResultOk: Sized + crate::private::Sealed {
    /// Turn the Ok value range (`(0,0x7FFFFFFF]`) into a option.
    ///
    /// The `ok_value` is always in the ok value range when used by [`SceResult`].
    ///
    /// # Safety
    /// [`SceResult`] methods that use it will ensure to pass only the valid range and the
    /// implementation is free to not check for values outside of that range.
    ///
    /// For callers outside of [`SceResult`], it must ensure to pass a valid `ok_value`.
    unsafe fn handle_ok_value(ok_value: u32) -> Option<Self>;

    /// Turn the Ok value range (`(0,0xFFFFFFFF_7FFFFFFF]`) into a option.
    ///
    /// The `ok_value` is always in the ok value range when used by [`SceResult64`].
    ///
    /// # Safety
    /// [`SceResult64`] methods that use it will ensure to pass only the valid range and the
    /// implementation is free to not check for values outside of that range.
    ///
    /// For callers outside of [`SceResult64`], it must ensure to pass a valid `ok_value`.
    unsafe fn handle_ok_value64(ok_value: u64) -> Option<Self> {
        match ok_value {
            0..=0x7FFFFFFF => unsafe { Self::handle_ok_value(ok_value as u32) },
            _ => None,
        }
    }
}

/// Trait that specifies that it can safely be turned to a `SceResult` ok-valid value.
///
/// # Safety
///
/// For this trait to be correct, the implementation must:
/// - Be valid for the `0..=0x7FFFFFFF` range
/// - Have a max size of 4 bytes
/// - If a struct, be `repr(transparent)`
pub unsafe trait SceIntoOkValue: Sized + crate::private::Sealed {
    fn into_ok_value(self) -> u32;
}

/// Trait that specifies that it can safely be turned to a `SceResult64` ok-valid value.
///
/// # Safety
///
/// For this trait to be correct, the implementation must:
/// - Be valid for the `0..=0x7FFFFFFF` range
/// - Have a max size of 4 bytes
/// - If a struct, be `repr(transparent)`
pub unsafe trait SceInto64OkValue: Sized + crate::private::Sealed {
    fn into_ok_value64(self) -> u64;
}

unsafe impl<T: SceIntoOkValue> SceInto64OkValue for T {
    fn into_ok_value64(self) -> u64 {
        self.into_ok_value() as u64
    }
}

macro_rules! __result_ok_int {
    ($($ty:ty),+) => {
        $(
            unsafe impl SceResultOk for $ty {
                unsafe fn handle_ok_value(ok_value: u32) -> Option<Self> {
                    <$ty>::try_from(ok_value).ok()
                }
            }

            unsafe impl SceIntoOkValue for $ty {
                fn into_ok_value(self) -> u32 {
                    self as u32
                }
            }
        )+
    };
}

__result_ok_int!(i8, u8, i16, u16, bool);


unsafe impl SceResultOk for i32 {
    unsafe fn handle_ok_value(ok_value: u32) -> Option<Self> {
        debug_assert!(ok_value <= 0x7FFFFFFF);
        Some(u32::cast_signed(ok_value))
    }
}
unsafe impl SceIntoOkValue for i32 {
    fn into_ok_value(self) -> u32 {
        self as u32
    }
}
unsafe impl SceResultOk for u32 {
    unsafe fn handle_ok_value(ok_value: u32) -> Option<Self> {
        debug_assert!(ok_value <= 0x7FFFFFFF);
        Some(ok_value)
    }
}
unsafe impl SceIntoOkValue for u32 {
    fn into_ok_value(self) -> u32 {
        self
    }
}
unsafe impl SceResultOk for i64 {
    unsafe fn handle_ok_value(ok_value: u32) -> Option<Self> {
        Some(ok_value as i64)
    }

    unsafe fn handle_ok_value64(ok_value: u64) -> Option<Self> {
        match ok_value {
            0..=0xFFFFFFFF_7FFFFFFF => Some(u64::cast_signed(ok_value)),
            _ => None,
        }
    }
}
unsafe impl SceResultOk for u64 {
    unsafe fn handle_ok_value(ok_value: u32) -> Option<Self> {
        Some(ok_value as u64)
    }

    unsafe fn handle_ok_value64(ok_value: u64) -> Option<Self> {
        match ok_value {
            0..=0xFFFFFFFF_7FFFFFFF => Some(ok_value),
            _ => None,
        }
    }
}
unsafe impl SceInto64OkValue for i64 {
    fn into_ok_value64(self) -> u64 {
        self as u64
    }
}
unsafe impl SceInto64OkValue for u64 {
    fn into_ok_value64(self) -> u64 {
        self
    }
}

#[cfg(target_pointer_width = "32")]
unsafe impl SceResultOk for isize {
    unsafe fn handle_ok_value(ok_value: u32) -> Option<Self> {
        debug_assert!(ok_value <= 0x7FFFFFFF);
        Some(ok_value as isize)
    }
}
#[cfg(target_pointer_width = "32")]
unsafe impl SceIntoOkValue for isize {
    fn into_ok_value(self) -> u32 {
        self as u32
    }
}
#[cfg(target_pointer_width = "32")]
unsafe impl SceResultOk for usize {
    unsafe fn handle_ok_value(ok_value: u32) -> Option<Self> {
        debug_assert!(ok_value <= 0x7FFFFFFF);
        Some(ok_value as usize)
    }
}
#[cfg(target_pointer_width = "32")]
unsafe impl SceIntoOkValue for usize {
    fn into_ok_value(self) -> u32 {
        self as u32
    }
}
unsafe impl SceResultOk for () {
    unsafe fn handle_ok_value(ok_value: u32) -> Option<Self> {
        match ok_value {
            0x00 => Some(()),
            _ => None,
        }
    }
}
unsafe impl SceIntoOkValue for () {
    fn into_ok_value(self) -> u32 {
        0
    }
}
unsafe impl SceResultOk for ! {
    unsafe fn handle_ok_value(_ok_value: u32) -> Option<Self> {
        None
    }
}
unsafe impl SceIntoOkValue for ! {
    fn into_ok_value(self) -> u32 {
        0
    }
}
unsafe impl SceResultOk for core::convert::Infallible {
    unsafe fn handle_ok_value(_: u32) -> Option<Self> {
        None
    }
}
unsafe impl SceResultOk for SceUid {
    unsafe fn handle_ok_value(ok_value: u32) -> Option<Self> {
        debug_assert!(ok_value <= 0x7FFFFFFF);
        // SAFETY: SceUid is always on the ok value range
        Some(unsafe { Self::from_raw_unchecked(ok_value) })
    }
}
unsafe impl SceIntoOkValue for SceUid {
    fn into_ok_value(self) -> u32 {
        self.to_inner()
    }
}

impl<T: SceResultOk + SceIntoOkValue> core::ops::Residual<T>
    for SceResult<core::convert::Infallible>
{
    type TryType = SceResult<T>;
}

impl<T: SceResultOk + SceIntoOkValue> core::ops::FromResidual for SceResult<T> {
    fn from_residual(residual: <Self as core::ops::Try>::Residual) -> Self {
        Self::new(residual.as_inner())
    }
}

impl<T: SceResultOk + SceIntoOkValue> core::ops::Try for SceResult<T> {
    type Output = T;
    type Residual = SceResult<core::convert::Infallible>;

    fn from_output(output: Self::Output) -> Self {
        SceResult::new(output.into_ok_value())
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self.into_result() {
            Ok(v) => ControlFlow::Continue(v),
            Err(err) => ControlFlow::Break(SceResult::new(err.to_inner())),
        }
    }
}

impl<T: SceResultOk + SceInto64OkValue> core::ops::Residual<T>
    for SceResult64<core::convert::Infallible>
{
    type TryType = SceResult64<T>;
}

impl<T: SceResultOk + SceInto64OkValue> core::ops::FromResidual for SceResult64<T> {
    fn from_residual(residual: <Self as core::ops::Try>::Residual) -> Self {
        Self::new(residual.as_inner())
    }
}

impl<T: SceResultOk + SceInto64OkValue> core::ops::Try for SceResult64<T> {
    type Output = T;
    type Residual = SceResult64<core::convert::Infallible>;

    fn from_output(output: Self::Output) -> Self {
        SceResult64::new(output.into_ok_value64())
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self.into_result() {
            Ok(v) => ControlFlow::Continue(v),
            Err(err) => ControlFlow::Break(SceResult64::new(err.to_inner() as u64)),
        }
    }
}

/// Resident/Stub library attributes.
///
/// Every library needs to have at least one of those attributes.
///
/// Resident libraries can have the members [`AutoExport`](LibFlags::AutoExport),
/// [`WeakExport`](LibFlags::WeakExport), [`NoLinkExport`](LibFlags::NoLinkExport),
/// [`SyscallExport`](LibFlags::SyscallExport) and [`IsSystemLib`](LibFlags::IsSystemLib).
///
/// Stub libraries can have [`NoSpecialFlags`](LibFlags::NoSpecialFlags) or
/// [`WeakImport`](LibFlags::WeakImport).
#[bitflag(u16)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum LibFlags {
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

#[macro_export]
#[doc(hidden)]
macro_rules! impl_ranged_ty {
    ($t:ty) => {
        impl ::core::marker::StructuralPartialEq for $t {}

        impl Eq for $t {}

        impl PartialEq for $t {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                self.to_inner() == other.to_inner()
            }
        }

        impl Ord for $t {
            #[inline]
            fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
                Ord::cmp(&self.to_inner(), &other.to_inner())
            }
        }

        impl PartialOrd for $t {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<::core::cmp::Ordering> {
                Some(Ord::cmp(self, other))
            }
        }

        impl ::core::hash::Hash for $t {
            // Required method
            fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                ::core::hash::Hash::hash(&self.to_inner(), state);
            }
        }
    };
}

/// Sets the processor K1 register to a given value.
///
/// This function is for use in kernel mode syscall exports. The kernel sets the `k1` register to
/// indicate what mode called the function, i.e. whether it was directly called, was called via a
/// syscall from a kernel thread or called via a syscall from a user thread. By setting `k1` to `0`
/// before doing anything in your code you can make the other functions think you are calling from a
/// kernel thread and therefore disable numerous protections. But it's usage must be careful, as the
/// some system API also checks for `k1` not being from the kernel to work properly.
///
/// # Safety
///
/// Touching a privileged CPU register is unsafe and changing K1 value can cause issues on system
/// state.
#[cfg(all(target_os = "psp", feature = "non-stub-code"))]
#[unsafe(naked)]
pub unsafe extern "C" fn set_k1(k1: u32) -> u32 {
    core::arch::naked_asm!(
        ".set noreorder",
        ".set noat",
        "move $v0, $k1",
        "jr	 $ra",
        "move $k1, $a0"
    )
}

/// Gets the current value of the processor K1 register.
#[cfg(all(target_os = "psp", feature = "non-stub-code"))]
#[unsafe(naked)]
pub extern "C" fn get_k1() -> u32 {
    core::arch::naked_asm!(".set noreorder", ".set noat", "jr $ra", "move $v0, $k1")
}

/// Disables the CPU FPU exceptions.
#[cfg(all(target_os = "psp", feature = "non-stub-code"))]
#[unsafe(naked)]
pub extern "C" fn disable_fpu_exceptions() {
    core::arch::naked_asm!(
        ".set noreorder",
        ".set noat",
        "cfc1 $2, $31",
        "lui $8, 0x80",
        "and $8, $2, $8",
        "ctc1 $8, $31",
        "jr $31",
        "nop",
    );
}

/// Disables interrupts, returning the previous state of the interrupt enable bit.
///
/// # Safety
///
/// Disabling interrupts for too long otherwise the watchdog will get you.
#[cfg(all(target_os = "psp", feature = "non-stub-code"))]
#[unsafe(naked)]
pub unsafe extern "C" fn suspend_interrupts() -> u32 {
    core::arch::naked_asm!(
        ".set noreorder",
        ".set noat",
        ".word 0x70020024", // mfic $v0, $0
        ".word 0x70000026", // mtic $0, $0
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "jr $31",
        "nop"
    )
}


/// Enables interrupts to the `state` value.
///
/// # Safety
///
/// The `state` must be a value returned by [`suspend_interrupts`].
#[cfg(all(target_os = "psp", feature = "non-stub-code"))]
#[unsafe(naked)]
pub unsafe extern "C" fn resume_interrupts(state: u32) {
    core::arch::naked_asm!(
        ".set noreorder",
        ".set noat",
        ".word 0x70040026", // mtic $a0, $0
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "jr $31",
        "nop",
    )
}

/// Disables interrupts, returning the previous state of the interrupt enable bit.
///
/// # Safety
///
/// Disabling interrupts for too long otherwise the watchdog will get you.
#[cfg(all(target_os = "psp", feature = "non-stub-code"))]
#[unsafe(naked)]
pub extern "C" fn get_current_interrupt_status() -> u32 {
    core::arch::naked_asm!(
        ".set noreorder",
        ".set noat",
        ".word 0x70020024", // mfic %0, $0
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "jr $31",
        "nop"
    )
}

/// Returns `true` if the the interrupt is enabled, `false` otherwise.
#[cfg(all(target_os = "psp", feature = "non-stub-code"))]
pub fn is_interrupt_enabled() -> bool {
    get_current_interrupt_status() != 0
}

/// Intrinsic to hint a spin-loop to the mips processor
///
/// Rust hint on mips is a no-op, doing nothing and making the loop hot.
#[cfg(all(target_os = "psp", feature = "non-stub-code"))]
pub fn spin_loop() {
    if is_interrupt_enabled() {
        let _ = thread::sceKernelDelayThread(1000);
    } else {
        core::hint::spin_loop();
    }
}

// SAFETY: must be called only once during runtime cleanup.
// NOTE: this is not guaranteed to run, for example when the program aborts.
#[cfg(all(target_os = "psp", feature = "non-stub-code"))]
pub(crate) unsafe fn cleanup() {}

/// Performs a volatile write of a memory location `addr` with the given `value` without
/// reading or dropping the old value.
///
/// Volatile operations are intended to act on I/O memory, and are guaranteed
/// to not be elided or reordered by the compiler across other volatile
/// operations.
///
/// # Safety
///
/// Behavior is undefined if any of the following conditions are violated:
///
/// * `addr` must be either valid for writes, or `addr` must point to memory outside of all Rust
///   allocations and writing to that memory must:
///   - not trap, and
///   - not cause any memory inside a Rust allocation to be modified.
///
/// * `addr` must be properly aligned.
#[track_caller]
#[inline(always)]
#[cfg(feature = "non-stub-code")]
pub unsafe fn volatile_write<T>(addr: usize, value: T)
where
    T: crate::private::VolatileOpAllowed,
{
    unsafe { core::ptr::with_exposed_provenance_mut::<T>(addr).write_volatile(value) };
}

/// Performs a volatile read of the value from `addr` without moving it. This
/// leaves the memory in `addr` unchanged.
///
/// Volatile operations are intended to act on I/O memory, and are guaranteed
/// to not be elided or reordered by the compiler across other volatile
/// operations.
///
/// # Safety
///
/// Behavior is undefined if any of the following conditions are violated:
///
/// * `addr` must be either valid for reads, or `addr` must point to memory outside of all Rust
///   allocations and reading from that memory must:
///   - not trap, and
///   - not cause any memory inside a Rust allocation to be modified.
///
/// * `addr` must be properly aligned.
#[track_caller]
#[inline(always)]
#[cfg(feature = "non-stub-code")]
pub unsafe fn volatile_read<T>(addr: usize) -> T
where
    T: crate::private::VolatileOpAllowed,
{
    unsafe { core::ptr::with_exposed_provenance_mut::<T>(addr).read_volatile() }
}

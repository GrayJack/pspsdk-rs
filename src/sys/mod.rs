use core::marker::PhantomData;

use bitflag_attr::bitflag;

#[doc(hidden)]
#[cfg(target_os = "psp")]
pub mod macro_helpers;


mod error;
pub use error::SceError;

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
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceResult<T>(u32, PhantomData<T>);

impl<T: SceResultOk> SceResult<T> {
    /// Create a new SceResult.
    pub const fn new(raw: u32) -> Self {
        SceResult(raw, PhantomData)
    }

    /// Turn the SceResult into a [`Result`] type.
    pub fn into_result(self) -> Result<T, SceError> {
        match self.0 {
            0..=0x7FFFFFFF => unsafe { T::handle_ok_value(self.0) },
            0x80000001..=0xFFFFFFFF => Err(unsafe { SceError::new_unchecked(self.0) }),
            0x80000000 => Err(SceError::INVALID_VALUE),
        }
    }
}

/// Trait of types that can be the ok result of [`SceResult`].
///
/// For this trait to be correct, the implementation must:
/// - Be valid for the `0..=0x7FFFFFFF` range
/// - Have a max size of 4 bytes
/// - If a struct, be `repr(transparent)`
pub unsafe trait SceResultOk: Sized + crate::private::Sealed {
    /// Turn the Ok value range (`(0,0x7FFFFFFF]`) into a result
    ///
    /// The `ok_value` is always in the ok value range when used by [`SceResult`].
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

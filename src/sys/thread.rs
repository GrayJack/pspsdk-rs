use crate::sys::{SceError, SceResultOk, SceUid};

/// The thread UID, created with [`sceKernelCreateThread`].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThreadId(SceUid);

/// The semaphore UID, created with [`sceKernelCreateSema`].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemaId(SceUid);

/// The event flag UID, created with [`sceKernelCreateEventFlag`].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventFlagId(SceUid);

/// The callback UID, created with [`sceKernelCreateCallback`].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CallbackId(SceUid);


impl ThreadId {
    /// Create a new thread ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceAtracId`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn new(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::new_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new thread ID structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn new_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::new_unchecked(raw) })
    }

    #[inline]
    pub const fn as_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for ThreadId {}
unsafe impl SceResultOk for ThreadId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe { SceUid::handle_ok_value(ok_value).map(Self) }
    }
}

impl SemaId {
    /// Create a new semaphore ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceAtracId`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn new(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::new_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new semaphore ID structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn new_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::new_unchecked(raw) })
    }

    #[inline]
    pub const fn as_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for SemaId {}
unsafe impl SceResultOk for SemaId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe { SceUid::handle_ok_value(ok_value).map(Self) }
    }
}

impl EventFlagId {
    /// Create a new event flag ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceAtracId`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn new(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::new_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new event flag ID structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn new_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::new_unchecked(raw) })
    }

    #[inline]
    pub const fn as_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for EventFlagId {}
unsafe impl SceResultOk for EventFlagId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe { SceUid::handle_ok_value(ok_value).map(Self) }
    }
}

impl CallbackId {
    /// Create a new callback ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceAtracId`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn new(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::new_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new callback ID structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn new_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::new_unchecked(raw) })
    }

    #[inline]
    pub const fn as_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for CallbackId {}
unsafe impl SceResultOk for CallbackId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe { SceUid::handle_ok_value(ok_value).map(Self) }
    }
}

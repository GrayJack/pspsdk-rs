//! Headphone Remote
#![allow(unused_imports)]

use bitflag_attr::bitflag;
use pspsdk_macros::psp_stub;

use crate::sys::{SceError, SceResult, SceResultOk, SceUid};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct HprmCallbackSlot(u32);

#[bitflag(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias("SceHprmKeys", "PspHprmKeys"))]
pub enum HprmKey {
    #[doc(alias("SCE_HPRM_PLAYPAUSE", "PSP_HPRM_PLAYPAUSE"))]
    PlayPause = 0x01,
    #[doc(alias("SCE_HPRM_FORWARD", "PSP_HPRM_FORWARD"))]
    Forward = 0x04,
    #[doc(alias("SCE_HPRM_BACK", "PSP_HPRM_BACK"))]
    Back = 0x08,
    #[doc(alias("SCE_HPRM_VOL_UP", "PSP_HPRM_VOL_UP"))]
    VolumeUp = 0x10,
    #[doc(alias("SCE_HPRM_VOL_DOWN", "PSP_HPRM_VOL_DOWN"))]
    VolumeDown = 0x20,
    #[doc(alias("SCE_HPRM_HOLD", "PSP_HPRM_HOLD"))]
    Hold = 0x80,
}

#[psp_stub(libname = "sceHprm", flags = 0x4001, use_crate)]
extern "C" {
    /// Peek at the current key being pressed on the remote.
    ///
    /// # Parameters
    ///
    /// - `key`: A reference to receive the key bitmap.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x1910B327)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceHprmPeekCurrentKey(key: &mut HprmKey) -> SceResult<()>;

    /// Peek at the current latch data.
    ///
    /// # Parameters
    ///
    /// - `latch`: A reference a to a 4 dword array to contain the latch data.
    ///
    /// # Return Value
    ///
    /// Unknown `Ok` value on success, error value otherwise.
    #[nid(0x2BCEC83E)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceHprmPeekLatch(latch: &mut [u32; 4]) -> SceResult<i32>;

    /// Read the current latch data.
    ///
    /// # Parameters
    ///
    /// - `latch`: A reference a to a 4 dword array to contain the latch data.
    ///
    /// # Return Value
    ///
    /// Unknown `Ok` value on success, error value otherwise.
    #[nid(0x40D2F9F0)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceHprmReadLatch(latch: &mut [u32; 4]) -> SceResult<i32>;

    /// Determines whether the headphones are plugged in.
    ///
    /// # Return Value
    ///
    /// Returns `true` if the headphones are plugged in, `false` otherwise.
    #[nid(0x7E69EDA4)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceHprmIsHeadphoneExist() -> bool;

    /// Determines whether the remote is plugged in.
    ///
    /// # Return Value
    ///
    /// Returns `true` if the remote is plugged in, `false` otherwise.
    #[nid(0x208DB1BD)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceHprmIsRemoteExist() -> bool;

    /// Determines whether the microphone is plugged in.
    ///
    /// # Return Value
    ///
    /// Returns `true` if the microphone is plugged in, `false` otherwise.
    #[nid(0x219C58F1)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceHprmIsMicrophoneExist() -> bool;

    /// Registers Hprm callback function.
    ///
    /// # Parameters
    /// - `slot`: The slot to register the callback function. Use [`HprmCallbackSlot::AVAILABLE`] to
    ///   register to a available slot and receive the slot value back.
    /// - `callback_id`: The callback ID from calling
    ///   [`sceKernelCreateCallback`](crate::thread::sceKernelCreateCallback).
    ///
    /// # Return Value
    ///
    /// If given `slot` is [`HprmCallbackSlot::AVAILABLE`], returns the slot the callback was
    /// registered on success, an error value otherwise.
    ///
    /// If given `slot` is a slot number, returns [`HprmCallbackSlot::ZERO`] on success, an error
    /// value otherwise.
    #[nid(0xC7154136)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceHprmRegisterCallback(
        slot: HprmCallbackSlot, callback_id: SceUid,
    ) -> SceResult<HprmCallbackSlot>;


    /// Unregisters Hprm callback function.
    ///
    /// # Parameters
    /// - `slot`: The slot to unregister the callback function.
    ///
    /// # Return Value
    /// `Ok` value on success, error value otherwise.
    #[nid(0x444ED0B7)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceHprmUnregitserCallback(slot: HprmCallbackSlot) -> SceResult<()>;
}

// FIXME: Add missing functions.
// FIXME: Add NID for the OFW range 3.70 to 3.73, but before that, we should find the version ranges
// that had no change in NIDs to reduce the number of needed features
#[cfg(feature = "kernel")]
#[psp_stub(libname = "sceHprm_driver", flags = 0x0001, use_crate)]
extern "C" {
    /// Initialize Headphone Remote module.
    #[nid(0x1C5BC5A0)]
    fn sceHprmInit();

    /// De-initialize Headphone Remote module.
    #[nid(0x588845DA)]
    fn sceHprmEnd();

    /// Suspends the headphone remote drive.
    #[nid(0x526BB7F4)]
    fn sceHprmSuspend();

    /// Resumes the headphone remote drive.
    #[nid(0x2C7B8B05)]
    fn sceHprmResume();

    /// Resets the headphone remote drive.
    #[nid(if cfg!(feature = "psp_660") { 0x1F64B227 } else { 0x2BCEC83E })]
    fn sceHprmReset();

    /// Peek at the current key being pressed on the remote.
    ///
    /// # Parameters
    ///
    /// - `key`: A reference to receive the key bitmap.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x1910B327)]
    pub fn sceHprmPeekCurrentKey(key: &mut HprmKey) -> SceResult<()>;

    /// Peek at the current latch data.
    ///
    /// # Parameters
    ///
    /// - `latch`: A reference a to a 4 dword array to contain the latch data.
    ///
    /// # Return Value
    ///
    /// Unknown `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x1F64B227 } else { 0x2BCEC83E })]
    pub fn sceHprmPeekLatch(latch: &mut [u32; 4]) -> SceResult<i32>;

    /// Read the current latch data.
    ///
    /// # Parameters
    ///
    /// - `latch`: A reference a to a 4 dword array to contain the latch data.
    ///
    /// # Return Value
    ///
    /// Unknown `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xE9B776BE } else { 0x40D2F9F0 })]
    pub fn sceHprmReadLatch(latch: &mut [u32; 4]) -> SceResult<i32>;

    /// Determines whether the headphones are plugged in.
    ///
    /// # Return Value
    ///
    /// Returns `true` if the headphones are plugged in, `false` otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xFA4A25A7 } else { 0x7E69EDA4 })]
    pub fn sceHprmIsHeadphoneExist() -> bool;

    /// Determines whether the remote is plugged in.
    ///
    /// # Return Value
    ///
    /// Returns `true` if the remote is plugged in, `false` otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xEFCFD0C5 } else { 0x208DB1BD })]
    pub fn sceHprmIsRemoteExist() -> bool;

    /// Determines whether the microphone is plugged in.
    ///
    /// # Return Value
    ///
    /// Returns `true` if the microphone is plugged in, `false` otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xAD158331 } else { 0x219C58F1 })]
    pub fn sceHprmIsMicrophoneExist() -> bool;

    /// Register Hprm callback function.
    ///
    /// # Parameters
    /// - `slot`: The slot to register the callback function. Use [`HprmCallbackSlot::AVAILABLE`] to
    ///   register to a available slot and receive the slot value back.
    /// - `callback_id`: The callback ID from calling
    ///   [`sceKernelCreateCallback`](crate::sys::thread::sceKernelCreateCallback).
    ///
    /// # Return Value
    ///
    /// If given `slot` is [`HprmCallbackSlot::AVAILABLE`], returns the slot the callback was
    /// registered on success, an error value otherwise.
    ///
    /// If given `slot` is a slot number, returns [`HprmCallbackSlot::ZERO`] on success, an error
    /// value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xDFC57C88 } else { 0xC7154136 })]
    pub fn sceHprmRegisterCallback(
        slot: HprmCallbackSlot, callback_id: SceUid,
    ) -> SceResult<HprmCallbackSlot>;

    /// Unregisters Hprm callback function.
    ///
    /// # Parameters
    /// - `slot`: The slot to unregister the callback function.
    ///
    /// # Return Value
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xEB0CFCCC } else { 0x444ED0B7 })]
    pub fn sceHprmUnregitserCallback(slot: HprmCallbackSlot) -> SceResult<()>;
}


impl HprmCallbackSlot {
    /// Callback slot to register the callback on available slot and receive the slot value back on
    /// [`sceHprmRegisterCallback`].
    pub const AVAILABLE: HprmCallbackSlot = unsafe { Self::from_raw_unchecked(0xFFFFFFFF) };
    /// Callback slot zero that also can be returned if passed [`HprmCallbackSlot::AVAILABLE`] to
    /// [`sceHprmRegisterCallback`] as success value.
    pub const ZERO: HprmCallbackSlot = unsafe { Self::from_raw_unchecked(0) };

    /// Create a new channel number from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceChannel`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            0u32..0x20 => Some(unsafe { Self::from_raw_unchecked(raw) }),
            0xFFFFFFFF => Some(Self::AVAILABLE),
            _ => None,
        }
    }

    /// Create a new channel number structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn from_raw_unchecked(raw: u32) -> Self {
        Self(raw)
    }

    #[inline]
    pub const fn to_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for HprmCallbackSlot {}
unsafe impl SceResultOk for HprmCallbackSlot {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        // On result Channel is never 0xFFFFFFFF
        match ok_value {
            0..0x20 => Ok(unsafe { Self::from_raw_unchecked(ok_value) }),
            _ => Err(SceError::INVALID_VALUE),
        }
    }
}

//! Headphone Remote

use bitflag_attr::bitflag;
use pspsdk_macros::psp_stub;

use crate::sys::{SceError, SceResult};

#[bitflag(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HprmKey {
    PlayPause = 0x01,
    Forward = 0x04,
    Back = 0x08,
    VolumeUp = 0x10,
    VolumeDown = 0x20,
    Hold = 0x80,
}

#[psp_stub(libname = "sceHprm", flags = 0x4001)]
extern "C" {
    /// Peek at the current key being pressed on the remote.
    ///
    /// # Parameters
    ///
    /// - `key`: A reference to receive the key bitmap.
    ///
    /// # Return Value
    ///
    /// [`Ok`] value on success, error value otherwise.
    #[cfg(not(feature = "kernel"))]
    pub fn sceHprmPeekCurrentKey(key: &mut HprmKey) -> Result<(), SceError>;

    /// Peek at the current latch data.
    ///
    /// # Parameters
    ///
    /// - `latch`: A reference a to a 4 dword array to contain the latch data.
    ///
    /// # Return Value
    ///
    /// Unknown `Ok` value on success, error value otherwise.
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
    #[cfg(not(feature = "kernel"))]
    pub fn sceHprmReadLatch(latch: &mut [u32; 4]) -> SceResult<i32>;

    /// Determines whether the headphones are plugged in.
    ///
    /// # Return Value
    ///
    /// Returns `true` if the headphones are plugged in, `false` otherwise.
    #[cfg(not(feature = "kernel"))]
    pub fn sceHprmIsHeadphoneExist() -> bool;

    /// Determines whether the remote is plugged in.
    ///
    /// # Return Value
    ///
    /// Returns `true` if the remote is plugged in, `false` otherwise.
    #[cfg(not(feature = "kernel"))]
    pub fn sceHprmIsRemoteExist() -> bool;

    /// Determines whether the microphone is plugged in.
    ///
    /// # Return Value
    ///
    /// Returns `true` if the microphone is plugged in, `false` otherwise.
    #[cfg(not(feature = "kernel"))]
    pub fn sceHprmIsMicrophoneExist() -> bool;
}

// FIXME: Add missing functions.
#[cfg(feature = "kernel")]
#[psp_stub(libname = "sceHprm_driver", flags = 0x0001)]
extern "C" {
    /// Initialize Headphone Remote module.
    fn sceHprmInit();

    /// De-initialize Headphone Remote module.
    fn sceHprmEnd();

    /// Suspends the headphone drive.
    fn sceHprmSuspend();

    /// Resumes the headphone drive.
    fn sceHprmResume();

    /// Peek at the current key being pressed on the remote.
    ///
    /// # Parameters
    ///
    /// - `key`: A reference to receive the key bitmap.
    ///
    /// # Return Value
    ///
    /// [`Ok`] value on success, error value otherwise.
    pub fn sceHprmPeekCurrentKey(key: &mut HprmKey) -> Result<(), SceError>;

    /// Peek at the current latch data.
    ///
    /// # Parameters
    ///
    /// - `latch`: A reference a to a 4 dword array to contain the latch data.
    ///
    /// # Return Value
    ///
    /// Unknown `Ok` value on success, error value otherwise.
    #[nid(if cfg!(any(feature = "vita_365", feature = "vita_epi")) { 0x1F64B227 } else { 0x2BCEC83E })]
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
    #[nid(if cfg!(any(feature = "vita_365", feature = "vita_epi")) { 0xE9B776BE } else { 0x40D2F9F0 })]
    pub fn sceHprmReadLatch(latch: &mut [u32; 4]) -> SceResult<i32>;

    /// Determines whether the headphones are plugged in.
    ///
    /// # Return Value
    ///
    /// Returns `true` if the headphones are plugged in, `false` otherwise.
    #[nid(if cfg!(any(feature = "vita_365", feature = "vita_epi")) { 0xFA4A25A7 } else { 0x7E69EDA4 })]
    pub fn sceHprmIsHeadphoneExist() -> bool;

    /// Determines whether the remote is plugged in.
    ///
    /// # Return Value
    ///
    /// Returns `true` if the remote is plugged in, `false` otherwise.
    #[nid(if cfg!(any(feature = "vita_365", feature = "vita_epi")) { 0xEFCFD0C5 } else { 0x208DB1BD })]
    pub fn sceHprmIsRemoteExist() -> bool;

    /// Determines whether the microphone is plugged in.
    ///
    /// # Return Value
    ///
    /// Returns `true` if the microphone is plugged in, `false` otherwise.
    #[nid(if cfg!(any(feature = "vita_365", feature = "vita_epi")) { 0xAD158331 } else { 0x219C58F1 })]
    pub fn sceHprmIsMicrophoneExist() -> bool;
}

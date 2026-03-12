//! Audio routing control
#![allow(unused_imports)]

use pspsdk_macros::psp_stub;

use crate::sys::{SceError, SceResult, SceResultOk};

/// The routing mode behavior.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum AudioRoutingMode {
    /// Audio output on speaker, automatically change to headphone when it is plugged.
    #[default]
    Auto = 0x00,
    /// Audio output on speaker, **do not** automatically change to headphone when it is plugged.
    PreferSpeakers = 0x01,
    /// Audio output only on speakers, always.
    OnlySpeakers = 0x02,
    /// Audio output only on headphone, always.
    OnlyHeadphone = 0x03,
}

/// The audio routing volume behavior mode.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum AudioRoutingVolumeMode {
    /// Normal. Follows the system volume.
    #[default]
    Normal = 0x00,
    /// Force maximum volume at all times.
    ForceMaximum = 0x01,
}

#[psp_stub(libname = "sceAudioRouting", flags = 0x4001, version = (0x00, 0x17))]
extern "C" {
    /// Gets the current audio routing mode.
    ///
    /// # Return Values
    ///
    /// Returns the current routing mode.
    #[nid(0x931ABEF5)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceAudioRoutingGetMode() -> AudioRoutingMode;

    /// Sets the audio routing mode.
    ///
    /// # Parameters
    ///
    /// - `mode`: The routing mode to set.
    ///
    /// # Return Values
    ///
    /// Returns the precious routing mode on success, an error value otherwise.
    #[nid(0x18B6F449)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceAudioRoutingSetMode(mode: AudioRoutingMode) -> SceResult<AudioRoutingMode>;

    /// Gets the current audio routing volume mode.
    ///
    /// # Return Value
    /// Returns the current routing volume mode.
    #[nid(0xD82D02FD)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceAudioRoutingGetVolumeMode() -> AudioRoutingVolumeMode;

    /// Sets the audio routing volume mode.
    ///
    /// # Parameters
    ///
    /// - `vol_mode`: The audio routing mode to set.
    ///
    /// # Return Value
    /// `Ok` value on success, error value otherwise.
    #[nid(0x44B384EF)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceAudioRoutingSetVolumeMode(vol_mode: AudioRoutingVolumeMode) -> Result<(), SceError>;
}

#[cfg(feature = "kernel")]
#[psp_stub(libname = "sceAudioRouting_driver", flags = 0x0001, version = (0x00, 0x17))]
extern "C" {
    /// Gets the current audio routing mode.
    ///
    /// # Return Values
    ///
    /// Returns the current routing mode.
    #[nid(if cfg!(feature = "psp_660") { 0x931ABEF5 }
        // else if cfg!(feature = "psp_630") { 0x39240E7D }
        // else if cfg!(feature = "psp_600") { 0x39240E7D }
        // else if cfg!(feature = "psp_570") { 0x39240E7D }
        else if cfg!(feature = "psp_500") { 0xE45BCCA6 }
        else if cfg!(feature = "psp_420") { 0x534B5BD7 }
        else if cfg!(feature = "psp_395") { 0xBFC44660 }
        else if cfg!(feature = "psp_380") { 0xF2EFEBB8 }
        else { 0x39240E7D }
    )]
    pub fn sceAudioRoutingGetMode() -> AudioRoutingMode;

    /// Sets the audio routing mode.
    ///
    /// # Parameters
    ///
    /// - `mode`: The routing mode to set.
    ///
    /// # Return Values
    ///
    /// Returns the precious routing mode on success, an error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x18B6F449 }
        // else if cfg!(feature = "psp_630") { 0x36FD8AA9 }
        // else if cfg!(feature = "psp_600") { 0x36FD8AA9 }
        // else if cfg!(feature = "psp_570") { 0x36FD8AA9 }
        else if cfg!(feature = "psp_500") { 0x758DF112 }
        else if cfg!(feature = "psp_420") { 0xEF3EA0FC }
        else if cfg!(feature = "psp_395") { 0x30237217 }
        else if cfg!(feature = "psp_380") { 0x414E7D7A }
        else { 0x36FD8AA9 }
    )]
    pub fn sceAudioRoutingSetMode(mode: AudioRoutingMode) -> SceResult<AudioRoutingMode>;

    /// Gets the current audio routing volume mode.
    ///
    /// # Return Value
    /// Returns the current routing volume mode.
    #[nid(if cfg!(feature = "psp_660") { 0xD82D02FD }
        // else if cfg!(feature = "psp_630") { 0x28235C56 }
        // else if cfg!(feature = "psp_600") { 0x28235C56 }
        // else if cfg!(feature = "psp_570") { 0x28235C56 }
        else if cfg!(feature = "psp_500") { 0x78DD0203 }
        else if cfg!(feature = "psp_420") { 0x6211AF01 }
        else if cfg!(feature = "psp_395") { 0x8C516FCA }
        else if cfg!(feature = "psp_380") { 0x1B372F9D }
        else { 0x28235C56 }
    )]
    pub fn sceAudioRoutingGetVolumeMode() -> AudioRoutingVolumeMode;

    /// Sets the audio routing volume mode.
    ///
    /// # Parameters
    ///
    /// - `vol_mode`: The audio routing mode to set.
    ///
    /// # Return Value
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x44B384EF }
        // else if cfg!(feature = "psp_630") { 0xBB548475 }
        // else if cfg!(feature = "psp_600") { 0xBB548475 }
        // else if cfg!(feature = "psp_570") { 0xBB548475 }
        else if cfg!(feature = "psp_500") { 0xB557A064 }
        else if cfg!(feature = "psp_420") { 0x3CF16BA0 }
        else if cfg!(feature = "psp_395") { 0x472E6A92 }
        else if cfg!(feature = "psp_380") { 0xBE9B6FF5 }
        else { 0xBB548475 }
    )]
    pub fn sceAudioRoutingSetVolumeMode(vol_mode: AudioRoutingVolumeMode) -> Result<(), SceError>;
}

impl crate::private::Sealed for AudioRoutingMode {}
unsafe impl SceResultOk for AudioRoutingMode {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        match ok_value {
            0x00 => Ok(Self::Auto),
            0x01 => Ok(Self::PreferSpeakers),
            0x02 => Ok(Self::OnlySpeakers),
            0x03 => Ok(Self::OnlyHeadphone),
            _ => Err(SceError::INVALID_VALUE),
        }
    }
}

impl crate::private::Sealed for AudioRoutingVolumeMode {}
unsafe impl SceResultOk for AudioRoutingVolumeMode {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        match ok_value {
            0x00 => Ok(Self::Normal),
            0x01 => Ok(Self::ForceMaximum),
            _ => Err(SceError::INVALID_VALUE),
        }
    }
}

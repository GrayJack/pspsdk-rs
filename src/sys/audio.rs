#![allow(unused_imports)]

use core::ffi::c_void;

use pspsdk_macros::psp_stub;

use crate::sys::{SceError, SceResult, SceResultOk};

pub mod routing;

/// Minimum value for audio sample value.
pub const AUDIO_SAMPLE_MIN: u32 = 64;
/// Maximum value for audio sample value.
pub const AUDIO_SAMPLE_MAX: u32 = 65472;

/// Representation of the PSP channel number.
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AudioChannelId(u32);

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias("SceAudioInputParams", "pspAudioInputParams"))]
pub struct AudioInputParams {
    /// Automatic Level Control (ALC) configuration.
    pub alto_level_control: i32,
    /// The input gain (amplification level).
    pub gain: i32,
    /// The noise gate/reduction threshold.
    pub noise: i32,
    /// The hold time before decay begins (in audio processing envelope).
    pub hold: i32,
    /// The decay rate (how quickly the level decreases).
    pub decay: i32,
    /// The attack rate (how quickly the level increases).
    pub attack: i32,
}

/// Possible audio formats for PSP.
#[repr(u32)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AudioFormats {
    /// Channel set to stereo output.
    #[doc(alias("PSP_AUDIO_FORMAT_STEREO"))]
    Stereo = 0,
    /// Channel set to mono output.
    #[doc(alias("PSP_AUDIO_FORMAT_MONO"))]
    Mono   = 0x10,
}

/// Possible values for Audio output frequency.
#[repr(u32)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AudioOutputFrequency {
    Khz48 = 48000,
    Khz44_1 = 44100,
    Khz32 = 32000,
    Khz24 = 24000,
    Khz22_05 = 22050,
    Khz16 = 16000,
    Khz12 = 12000,
    Khz11_025 = 11025,
    Khz8  = 8000,
}

/// Possible values for Audio input frequency.
#[repr(u32)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AudioInputFrequency {
    Khz44_1 = 44100,
    Khz22_05 = 22050,
    Khz11_025 = 11025,
}

#[psp_stub(libname = "sceAudio", flags = 0x4001, use_crate)]
extern "C" {
    /// Allocate and initialize a hardware output channel.
    ///
    /// # Parameters
    ///
    /// - `channel`: The channel ID to reserve. Pass [`AudioChannelId::NEXT`] to get the first
    ///   available channel.
    /// - `sample_count`: The number of samples that can be output on the channel per output call.
    ///   It must be a value between [`AUDIO_SAMPLE_MIN`] and [`AUDIO_SAMPLE_MAX`], and it must be
    ///   aligned to 64 bytes. Use [`audio_sample_align`] to align it.
    /// - `format`: The output format to use for the channel.
    ///
    /// # Return value
    ///
    /// The channel ID on success, an error value otherwise.
    #[nid(0x5EC81C55)]
    pub fn sceAudioChReserve(
        channel: AudioChannelId, sample_count: i32, format: AudioFormats,
    ) -> SceResult<AudioChannelId>;
}

/// Make the given sample count a multiple of 64.
pub const fn audio_sample_align(sample_count: i32) -> i32 {
    (sample_count + 63) & !63
}

impl AudioChannelId {
    /// Channel 0
    pub const CHANNEL_0: AudioChannelId = unsafe { Self::from_raw_unchecked(0) };
    /// Channel 1
    pub const CHANNEL_1: AudioChannelId = unsafe { Self::from_raw_unchecked(1) };
    /// Channel 2
    pub const CHANNEL_2: AudioChannelId = unsafe { Self::from_raw_unchecked(2) };
    /// Channel 3
    pub const CHANNEL_3: AudioChannelId = unsafe { Self::from_raw_unchecked(3) };
    /// Channel 4
    pub const CHANNEL_4: AudioChannelId = unsafe { Self::from_raw_unchecked(4) };
    /// Channel 5
    pub const CHANNEL_5: AudioChannelId = unsafe { Self::from_raw_unchecked(5) };
    /// Channel 6
    pub const CHANNEL_6: AudioChannelId = unsafe { Self::from_raw_unchecked(6) };
    /// Channel 7
    pub const CHANNEL_7: AudioChannelId = unsafe { Self::from_raw_unchecked(7) };
    /// Channel value to request a the next channel on functions like [`sceAudioChReserve`].
    pub const NEXT: AudioChannelId = unsafe { Self::from_raw_unchecked(0xFFFFFFFF) };

    /// Create a new channel number from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceChannel`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            0u32..=7 => Some(unsafe { Self::from_raw_unchecked(raw) }),
            0xFFFFFFFF => Some(Self::NEXT),
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
    pub const fn as_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for AudioChannelId {}
unsafe impl SceResultOk for AudioChannelId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        // On result Channel is never 0xFFFFFFFF
        match ok_value {
            0..=7 => Ok(unsafe { Self::from_raw_unchecked(ok_value) }),
            _ => Err(SceError::INVALID_VALUE),
        }
    }
}

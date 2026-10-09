//! Audio operation and management.
#![allow(unused_imports)]

use core::ffi::c_void;

use pspsdk_macros::{psp_fw_cfg, psp_stub};

use crate::sys::{SceError, SceIntoOkValue, SceResult, SceResultOk, SceSize};

pub mod atrac;
pub mod routing;

/// Minimum value for audio sample value.
pub const AUDIO_SAMPLE_MIN: u32 = 64;
/// Maximum value for audio sample value.
pub const AUDIO_SAMPLE_MAX: u32 = 65472;

/// Representation of the PSP channel number.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AudioChannelId(u32);

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias("SceAudioInputParams", "pspAudioInputParams"))]
pub struct AudioInputParams {
    /// Automatic Level Control (ALC) configuration.
    ///
    /// Valid range: `-29` to `0`.
    pub alto_level_control: i32,
    /// The input gain (amplification level).
    ///
    /// Valid range: `-18` to `30`.
    pub gain: i32,
    /// The noise gate/reduction threshold.
    ///
    /// Valid range: `-76` to `0`.
    pub noise: i32,
    /// The hold time before decay begins (in audio processing envelope).
    ///
    /// Valid range: `0` to `15`.
    pub hold: i32,
    /// The decay rate (how quickly the level decreases).
    ///
    /// Valid range: `0` to `10`.
    pub decay: i32,
    /// The attack rate (how quickly the level increases).
    ///
    /// Valid range: `0` to `10`.
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

/// Possible values for number of audio channels.
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum AudioChannels {
    /// One channel.
    Mono   = 1,
    /// Two channels.
    #[default]
    Stereo = 2,
}

/// The audio input completion status returned by [`sceAudioPollInputEnd`].
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum AudioInputCompletionStatus {
    /// The audio input was completed.
    #[default]
    Complete = 0,
    /// The audio input is not complete yet.
    NotComplete = 1,
}

#[psp_stub(libname = "sceAudio", flags = 0x4009, use_crate)]
unsafe extern "C" {
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
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceAudioChReserve(
        channel: AudioChannelId, sample_count: i32, format: AudioFormats,
    ) -> SceResult<AudioChannelId>;

    /// Releases an initialized audio channel ID.
    ///
    /// # Parameters
    ///
    /// - `channel`: The channel ID to release.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x6FC46853)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceAudioChRelease(channel: AudioChannelId) -> SceResult<()>;

    /// Gets the output audio of a channel.
    ///
    /// # Parameters
    ///
    /// - `channel`: The channel ID.
    /// - `volume`: The volume of the audio
    /// - `buf` **[[Out parameter]]**: The buffer to receive the PCM audio output.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x8C1009B2)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceAudioOutput(
        channel: AudioChannelId, volume: u32, buf: *mut u8,
    ) -> SceResult<()>;

    /// Gets the output audio of a channel (blocking).
    ///
    /// That means this functions returns immediately with an error if not able to get the output
    /// audio.
    ///
    /// # Parameters
    ///
    /// - `channel`: The channel ID.
    /// - `volume`: The volume of the audio
    /// - `buf` **[[Out parameter]]**: The buffer to receive the PCM audio output.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x136CAF51)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceAudioOutputBlocking(
        channel: AudioChannelId, volume: u32, buf: *mut u8,
    ) -> SceResult<()>;

    /// Gets the panned output audio of a channel asynchronously.
    ///
    /// # Parameters
    ///
    /// - `channel`: The channel ID.
    /// - `left_volume`: The left side volume of the audio
    /// - `right_volume`: The right side volume of the audio
    /// - `buf` **[[Out parameter]]**: The buffer to receive the PCM audio output.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xE2D56B2D)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceAudioOutputPanned(
        channel: AudioChannelId, left_volume: u32, right_volume: u32, buf: *mut u8,
    ) -> SceResult<()>;

    /// Gets the count of unplayed samples remaining.
    ///
    /// # Parameters
    ///
    /// - `channel`: The channel ID.
    ///
    /// # Return Value
    ///
    /// Returns the count of unplayed samples remaining on success, error value otherwise.
    #[nid(0xE9D97901)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceAudioGetChannelRestLen(channel: AudioChannelId) -> SceResult<SceSize>;

    /// Gets the count of unplayed samples remaining.
    ///
    /// # Parameters
    ///
    /// - `channel`: The channel ID.
    ///
    /// # Return Value
    ///
    /// Returns the count of unplayed samples remaining on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[psp_fw_cfg(150..)]
    #[nid(0xB011922F)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceAudioGetChannelRestLength(channel: AudioChannelId) -> SceResult<SceSize>;

    /// Sets the output sample count, after it's already been reserved.
    ///
    /// # Parameters
    ///
    /// - `channel`: The channel ID.
    /// - `sample_count`: The number of samples to output in one output call to set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xCB2E439E)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceAudioSetChannelDataLen(channel: AudioChannelId, sample_count: i32) -> SceResult<()>;

    /// Changes the audio format of a channel.
    ///
    /// # Parameters
    ///
    /// - `channel`: The channel ID.
    /// - `format`: The audio format to set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x95FD0C2D)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceAudioChangeChannelConfig(
        channel: AudioChannelId, format: AudioFormats,
    ) -> SceResult<()>;

    /// Changes the volume of a channel.
    ///
    /// # Parameters
    ///
    /// - `channel`: The channel ID.
    /// - `left_volume`: The left side volume to set.
    /// - `right_volume`: The right side volume to set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xB7E1D8E7)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceAudioChangeChannelVolume(
        channel: AudioChannelId, left_volume: u32, right_volume: u32,
    ) -> SceResult<()>;

    /// Reserves the audio output and set the output sample count.
    ///
    /// # Parameters
    ///
    /// - `sample_count`: The number of samples to output in one output call (min 17, max 4111).
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[psp_fw_cfg(200..)]
    #[nid(0x01562BA3)]
    pub safe fn sceAudioOutput2Reserve(sample_count: u32) -> SceResult<()>;

    /// Releases the audio output and process termination.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[psp_fw_cfg(200..)]
    #[nid(0x43196845)]
    pub safe fn sceAudioOutput2Release() -> SceResult<()>;

    /// Change the output sample count after it's already been reserved.
    ///
    /// # Parameters
    ///
    /// - `sample_count`: The number of samples to output in one output call (min 17, max 4111).
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[psp_fw_cfg(200..)]
    #[nid(0x63F2889C)]
    pub safe fn sceAudioOutput2ChangeLength(sample_count: u32) -> SceResult<()>;

    /// Gets the output audio of a channel (blocking).
    ///
    /// That means this functions returns immediately with an error if not able to get the output
    /// audio.
    ///
    /// # Parameters
    ///
    /// - `volume`: The volume of the audio
    /// - `buf` **[[Out parameter]]**: The buffer to receive the PCM audio output.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[psp_fw_cfg(200..)]
    #[nid(0x2D53F36E)]
    pub safe fn sceAudioOutput2OutputBlocking(volume: u32, buf: *mut u8) -> SceResult<()>;

    /// Gets the count of unplayed samples remaining.
    ///
    /// # Return Value
    ///
    /// Returns the count of unplayed samples remaining on success, error value otherwise.
    #[psp_fw_cfg(200..)]
    #[nid(0x2D53F36E)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceAudioOutput2GetRestSample() -> SceResult<u32>;

    /// Reserves the audio output.
    ///
    /// # Parameters
    ///
    /// - `sample_count`: The number of samples to output in one output call (min 17, max 4111).
    /// - `frequency`: The output frequency to use.
    /// - `channels`: The number of channels to use.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x38553111)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceAudioSRCChReserve(
        sample_count: u32, frequency: AudioOutputFrequency, channels: AudioChannels,
    ) -> SceResult<()>;

    /// Releases the audio output.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x5C37C0AE)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceAudioSRCChRelease() -> SceResult<()>;

    /// Initializes the audio input.
    ///
    /// # Parameters
    ///
    /// - `alto_level_control`: Automatic Level Control (ALC) configuration. Valid range: `-29` to
    ///   `0`.
    /// - `gain`: The input gain (amplification level). Valid range: `-18` to `30`.
    /// - `noise`: The noise gate/reduction threshold. Valid range: `-76` to `0`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.03.
    #[nid(0x7DE61688)]
    #[psp_fw_cfg(103..)]
    #[cfg(not(feature = "kernel"))]
    #[deprecated(note = "replaced by `sceAudioInputInitEx`")]
    pub safe fn sceAudioInputInit(alto_level_control: i32, gain: i32, noise: i32) -> SceResult<()>;

    /// Initializes the audio input.
    ///
    /// # Parameters
    ///
    /// - `options` **[[In parameter]]**: The options to be used on initialization.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[psp_fw_cfg(150..)]
    #[nid(0xE926D3FB)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceAudioInputInitEx(options: &AudioInputParams) -> SceResult<()>;

    /// Performs audio input asynchronously.
    ///
    /// # Parameters
    ///
    /// - `sample_count`: The number of input samples.
    /// - `frequency`: The audio input frequency.
    /// - `buf` **[[Out parameter]]**: The buffer to store the audio input data.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x6D4BEC68)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceAudioInput(
        sample_count: u32, frequency: AudioInputFrequency, buf: *mut u8,
    ) -> SceResult<()>;

    /// Performs audio input (blocking).
    ///
    /// That means this functions returns immediately with an error if not able to get the input
    /// audio.
    ///
    /// # Parameters
    ///
    /// - `sample_count`: The number of input samples.
    /// - `frequency`: The audio input frequency.
    /// - `buf` **[[Out parameter]]**: The buffer to store the audio input data.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x086E5895)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceAudioInputBlocking(
        sample_count: u32, frequency: AudioInputFrequency, buf: *mut u8,
    ) -> SceResult<()>;

    /// Gets the number of samples that were acquired.
    ///
    /// # Return Value
    ///
    /// Returns the number of samples acquired on success, error value otherwise.
    #[nid(0xA708C6A6)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceAudioGetInputLength() -> SceResult<u32>;

    /// Polls the completion status for non-blocking audio input.
    ///
    /// # Return Value
    ///
    /// Returns the audio input completion status on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[psp_fw_cfg(150..)]
    #[nid(0xA633048E)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceAudioPollInputEnd() -> SceResult<AudioInputCompletionStatus>;
}

#[cfg(feature = "kernel")]
#[psp_stub(libname = "sceAudio_driver", flags = 0x0009, use_crate)]
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
    #[nid(if cfg!(feature = "psp_660") { 0x4F959094 }
        else if cfg!(feature = "psp_630") { 0xF84226FB }
        else if cfg!(feature = "psp_600") { 0x64B9E089 }
        else if cfg!(feature = "psp_570") { 0xBD758B3B }
        else if cfg!(feature = "psp_500") { 0x10E94C2E }
        else if cfg!(feature = "psp_420") { 0xC08F9F54 }
        else if cfg!(feature = "psp_395") { 0x16B6C3AC }
        else if cfg!(feature = "psp_380") { 0x6985717B }
        else if cfg!(feature = "psp_370") { 0xE4A9D621 }
        else { 0x5EC81C55 }
    )]
    pub safe fn sceAudioChReserve(
        channel: AudioChannelId, sample_count: i32, format: AudioFormats,
    ) -> SceResult<AudioChannelId>;

    /// Releases an initialized audio channel ID.
    ///
    /// # Parameters
    ///
    /// - `channel`: The channel ID to release.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xDA16FF61 }
        else if cfg!(feature = "psp_630") { 0xA6FD1A21 }
        else if cfg!(feature = "psp_600") { 0xA05BAA21 }
        else if cfg!(feature = "psp_570") { 0xB70C9449 }
        else if cfg!(feature = "psp_500") { 0x31DB12BC }
        else if cfg!(feature = "psp_420") { 0x24AD0624 }
        else if cfg!(feature = "psp_395") { 0x32228C37 }
        else if cfg!(feature = "psp_380") { 0x3E3C44F1 }
        else if cfg!(feature = "psp_370") { 0xC2031226 }
        else { 0x6FC46853 }
    )]
    pub unsafe fn sceAudioChRelease(channel: AudioChannelId) -> SceResult<()>;

    /// Gets the output audio of a channel asynchronously.
    ///
    /// # Parameters
    ///
    /// - `channel`: The channel ID.
    /// - `volume`: The volume of the audio
    /// - `buf` **[[Out parameter]]**: The buffer to receive the PCM audio output.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x671E97E8 }
        else if cfg!(feature = "psp_630") { 0xD35EFCD9 }
        else if cfg!(feature = "psp_600") { 0xA5C0603B }
        else if cfg!(feature = "psp_570") { 0x0A4450DB }
        else if cfg!(feature = "psp_500") { 0x324BA73D }
        else if cfg!(feature = "psp_420") { 0x960A4530 }
        else if cfg!(feature = "psp_395") { 0xF7083625 }
        else if cfg!(feature = "psp_380") { 0x2047EAD8 }
        else if cfg!(feature = "psp_370") { 0x035C14C6 }
        else { 0x8C1009B2 }
    )]
    pub unsafe fn sceAudioOutput(
        channel: AudioChannelId, volume: u32, buf: *mut u8,
    ) -> SceResult<()>;

    /// Gets the output audio of a channel (blocking).
    ///
    /// That means this functions returns immediately with an error if not able to get the output
    /// audio.
    ///
    /// # Parameters
    ///
    /// - `channel`: The channel ID.
    /// - `volume`: The volume of the audio
    /// - `buf` **[[Out parameter]]**: The buffer to receive the PCM audio output.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x5CDEF9A4 }
        else if cfg!(feature = "psp_630") { 0xA906D208 }
        else if cfg!(feature = "psp_600") { 0x2FD643D0 }
        else if cfg!(feature = "psp_570") { 0xD3D66AB8 }
        else if cfg!(feature = "psp_500") { 0xF355A263 }
        else if cfg!(feature = "psp_420") { 0xD42588C3 }
        else if cfg!(feature = "psp_395") { 0x61BBE62D }
        else if cfg!(feature = "psp_380") { 0xC66BDCA1 }
        else if cfg!(feature = "psp_370") { 0x798FB2A3 }
        else { 0x136CAF51 }
    )]
    pub unsafe fn sceAudioOutputBlocking(
        channel: AudioChannelId, volume: u32, buf: *mut u8,
    ) -> SceResult<()>;

    /// Gets the panned output audio of a channel asynchronously.
    ///
    /// # Parameters
    ///
    /// - `channel`: The channel ID.
    /// - `left_volume`: The left side volume of the audio
    /// - `right_volume`: The right side volume of the audio
    /// - `buf` **[[Out parameter]]**: The buffer to receive the PCM audio output.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x17D856B9 }
        else if cfg!(feature = "psp_630") { 0xC847973E }
        else if cfg!(feature = "psp_600") { 0x548EEFD1 }
        else if cfg!(feature = "psp_570") { 0xF66A2442 }
        else if cfg!(feature = "psp_500") { 0xE8D4EE26 }
        else if cfg!(feature = "psp_420") { 0x04D5E360 }
        else if cfg!(feature = "psp_395") { 0x850A552B }
        else if cfg!(feature = "psp_380") { 0xBE6F1721 }
        else if cfg!(feature = "psp_370") { 0x47D06FE6 }
        else { 0xE2D56B2D }
    )]
    pub unsafe fn sceAudioOutputPanned(
        channel: AudioChannelId, left_volume: u32, right_volume: u32, buf: *mut u8,
    ) -> SceResult<()>;

    /// Gets the panned output audio of a channel (blocking).
    ///
    /// That means this functions returns immediately with an error if not able to get the output
    /// audio.
    ///
    /// # Parameters
    ///
    /// - `channel`: The channel ID.
    /// - `left_volume`: The left side volume of the audio
    /// - `right_volume`: The right side volume of the audio
    /// - `buf` **[[Out parameter]]**: The buffer to receive the PCM audio output.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xB7CCF1D7 }
        else if cfg!(feature = "psp_630") { 0xB29D727F }
        else if cfg!(feature = "psp_600") { 0x23F19798 }
        else if cfg!(feature = "psp_570") { 0xBA830657 }
        else if cfg!(feature = "psp_500") { 0xDC34AC20 }
        else if cfg!(feature = "psp_420") { 0xBF50BB5B }
        else if cfg!(feature = "psp_395") { 0xC4839793 }
        else if cfg!(feature = "psp_380") { 0x0B61478E }
        else if cfg!(feature = "psp_370") { 0x5140D94F }
        else { 0x13F592BC }
    )]
    pub unsafe fn sceAudioOutputPannedBlocking(
        channel: AudioChannelId, left_volume: u32, right_volume: u32, buf: *mut u8,
    ) -> SceResult<()>;

    /// Gets the count of unplayed samples remaining.
    ///
    /// # Parameters
    ///
    /// - `channel`: The channel ID.
    ///
    /// # Return Value
    ///
    /// Returns the count of unplayed samples remaining on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xCF62029E }
        else if cfg!(feature = "psp_630") { 0x52F5804E }
        else if cfg!(feature = "psp_600") { 0x79AF2DA3 }
        else if cfg!(feature = "psp_570") { 0x8D8D8D28 }
        else if cfg!(feature = "psp_500") { 0xC7D0C9AA }
        else if cfg!(feature = "psp_420") { 0x415C02D3 }
        else if cfg!(feature = "psp_395") { 0x47A07980 }
        else if cfg!(feature = "psp_380") { 0xAD7A0669 }
        else if cfg!(feature = "psp_370") { 0xD81BCF3F }
        else { 0xE9D97901 }
    )]
    pub fn sceAudioGetChannelRestLen(channel: AudioChannelId) -> SceResult<SceSize>;

    /// Gets the count of unplayed samples remaining.
    ///
    /// # Parameters
    ///
    /// - `channel`: The channel ID.
    ///
    /// # Return Value
    ///
    /// Returns the count of unplayed samples remaining on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[psp_fw_cfg(150..)]
    #[nid(if cfg!(feature = "psp_660") { 0x9D77949E }
        else if cfg!(feature = "psp_630") { 0xA388ABDB }
        else if cfg!(feature = "psp_600") { 0x77EBE4A2 }
        else if cfg!(feature = "psp_570") { 0xE39449B6 }
        else if cfg!(feature = "psp_500") { 0x322078A5 }
        else if cfg!(feature = "psp_420") { 0x8809D0CD }
        else if cfg!(feature = "psp_395") { 0xFCC526B7 }
        else if cfg!(feature = "psp_380") { 0x80BCF50C }
        else if cfg!(feature = "psp_370") { 0xB282F4B2 }
        else { 0xB011922F }
    )]
    pub fn sceAudioGetChannelRestLength(channel: AudioChannelId) -> SceResult<SceSize>;

    /// Sets the output sample count, after it's already been reserved.
    ///
    /// # Parameters
    ///
    /// - `channel`: The channel ID.
    /// - `sample_count`: The number of samples to output in one output call to set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x093C7A46 }
        else if cfg!(feature = "psp_630") { 0x3F4D81C1 }
        else if cfg!(feature = "psp_600") { 0x6F968893 }
        else if cfg!(feature = "psp_570") { 0xB212064D }
        else if cfg!(feature = "psp_500") { 0xC9DEA954 }
        else if cfg!(feature = "psp_420") { 0x8C5C6807 }
        else if cfg!(feature = "psp_395") { 0x52E455A3 }
        else if cfg!(feature = "psp_380") { 0x9DF11DD1 }
        else if cfg!(feature = "psp_370") { 0xA17F55D7 }
        else { 0xCB2E439E }
    )]
    pub fn sceAudioSetChannelDataLen(channel: AudioChannelId, sample_count: i32) -> SceResult<()>;

    /// Changes the audio format of a channel.
    ///
    /// # Parameters
    ///
    /// - `channel`: The channel ID.
    /// - `format`: The audio format to set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x63B5A460 }
        else if cfg!(feature = "psp_630") { 0x5A0397C5 }
        else if cfg!(feature = "psp_600") { 0xCD62C216 }
        else if cfg!(feature = "psp_570") { 0xDA692AA3 }
        else if cfg!(feature = "psp_500") { 0x7404CF0E }
        else if cfg!(feature = "psp_420") { 0x8B78CE95 }
        else if cfg!(feature = "psp_395") { 0x027E9E8B }
        else if cfg!(feature = "psp_380") { 0xB6E3C6DB }
        else if cfg!(feature = "psp_370") { 0xE73CE6EE }
        else { 0x95FD0C2D }
    )]
    pub fn sceAudioChangeChannelConfig(
        channel: AudioChannelId, format: AudioFormats,
    ) -> SceResult<()>;

    /// Changes the volume of a channel.
    ///
    /// # Parameters
    ///
    /// - `channel`: The channel ID.
    /// - `left_volume`: The left side volume to set.
    /// - `right_volume`: The right side volume to set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xFAD7B114 }
        else if cfg!(feature = "psp_630") { 0xA25A3346 }
        else if cfg!(feature = "psp_600") { 0x57AB835D }
        else if cfg!(feature = "psp_570") { 0xF0A71012 }
        else if cfg!(feature = "psp_500") { 0xDEF3E3AF }
        else if cfg!(feature = "psp_420") { 0xDA9B8B7A }
        else if cfg!(feature = "psp_395") { 0xB99D064F }
        else if cfg!(feature = "psp_380") { 0x20856A07 }
        else if cfg!(feature = "psp_370") { 0xF9504CA4 }
        else { 0xB7E1D8E7 }
    )]
    pub fn sceAudioChangeChannelVolume(
        channel: AudioChannelId, left_volume: u32, right_volume: u32,
    ) -> SceResult<()>;

    /// Gets the count of unplayed samples remaining.
    ///
    /// # Return Value
    ///
    /// Returns the count of unplayed samples remaining on success, error value otherwise.
    #[psp_fw_cfg(200..)]
    #[nid(if cfg!(feature = "psp_660") { 0x8A7CD9C6 }
        // else if cfg!(feature = "psp_630") { 0x647CEF33 }
        // else if cfg!(feature = "psp_600") { 0x647CEF33 }
        // else if cfg!(feature = "psp_570") { 0x647CEF33 }
        // else if cfg!(feature = "psp_500") { 0x647CEF33 }
        // else if cfg!(feature = "psp_420") { 0x647CEF33 }
        // else if cfg!(feature = "psp_395") { 0x647CEF33 }
        // else if cfg!(feature = "psp_380") { 0x647CEF33 }
        // else if cfg!(feature = "psp_370") { 0x647CEF33 }
        else { 0x647CEF33 }
    )]
    pub safe fn sceAudioOutput2GetRestSample() -> SceResult<u32>;

    /// Reserves the audio output.
    ///
    /// # Parameters
    ///
    /// - `sample_count`: The number of samples to output in one output call (min 17, max 4111).
    /// - `frequency`: The output frequency to use.
    /// - `channels`: The number of channels to use.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x837701CC }
        else if cfg!(feature = "psp_630") { 0xE9BCD519 }
        else if cfg!(feature = "psp_600") { 0xA502088E }
        else if cfg!(feature = "psp_570") { 0xBD58E6BC }
        else if cfg!(feature = "psp_500") { 0xBFC404F6 }
        else if cfg!(feature = "psp_420") { 0x7D47BCCE }
        else if cfg!(feature = "psp_395") { 0xC15BE9A3 }
        else if cfg!(feature = "psp_380") { 0x4036F0AE }
        else if cfg!(feature = "psp_370") { 0x669D93E4 }
        else { 0x38553111 }
    )]
    pub safe fn sceAudioSRCChReserve(
        sample_count: u32, frequency: AudioOutputFrequency, channels: AudioChannels,
    ) -> SceResult<()>;

    /// Releases the audio output.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xB7F5A1B2 }
        else if cfg!(feature = "psp_630") { 0x0399579B }
        else if cfg!(feature = "psp_600") { 0xF462A7B9 }
        else if cfg!(feature = "psp_570") { 0x0363F674 }
        else if cfg!(feature = "psp_500") { 0x3E3FAE75 }
        else if cfg!(feature = "psp_420") { 0x0CCCF5A6 }
        else if cfg!(feature = "psp_395") { 0xC3C2C4B4 }
        else if cfg!(feature = "psp_380") { 0xD4828217 }
        else if cfg!(feature = "psp_370") { 0x138A70F1 }
        else { 0x5C37C0AE }
    )]
    pub safe fn sceAudioSRCChRelease() -> SceResult<()>;

    /// Initializes the audio input.
    ///
    /// # Parameters
    ///
    /// - `alto_level_control`: Automatic Level Control (ALC) configuration. Valid range: `-29` to
    ///   `0`.
    /// - `gain`: The input gain (amplification level). Valid range: `-18` to `30`.
    /// - `noise`: The noise gate/reduction threshold. Valid range: `-76` to `0`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.03.
    #[psp_fw_cfg(103..)]
    #[nid(if cfg!(feature = "psp_660") { 0x10900742 }
        else if cfg!(feature = "psp_630") { 0x2E5E3227 }
        else if cfg!(feature = "psp_600") { 0xB82D416A }
        else if cfg!(feature = "psp_570") { 0x67FD980D }
        else if cfg!(feature = "psp_500") { 0xC7A7DBB0 }
        else if cfg!(feature = "psp_420") { 0x12454D49 }
        else if cfg!(feature = "psp_395") { 0xE560A6B3 }
        else if cfg!(feature = "psp_380") { 0x9C56FF57 }
        else if cfg!(feature = "psp_370") { 0xCC966C3A }
        else { 0x7DE61688 }
    )]
    #[deprecated(note = "replaced by `sceAudioInputInitEx`")]
    pub safe fn sceAudioInputInit(alto_level_control: i32, gain: i32, noise: i32) -> SceResult<()>;

    /// Initializes the audio input.
    ///
    /// # Parameters
    ///
    /// - `options` **[[In parameter]]**: The options to be used on initialization.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[psp_fw_cfg(150..)]
    #[nid(if cfg!(feature = "psp_660") { 0x4AFA04D9 }
        // else if cfg!(feature = "psp_630") { 0xE926D3FB }
        // else if cfg!(feature = "psp_600") { 0xE926D3FB }
        // else if cfg!(feature = "psp_570") { 0xE926D3FB }
        else if cfg!(feature = "psp_500") { 0xA3A683FC }
        else if cfg!(feature = "psp_420") { 0xDD7D024E }
        else if cfg!(feature = "psp_395") { 0x6EE7810A }
        else if cfg!(feature = "psp_380") { 0x9BD06636 }
        else if cfg!(feature = "psp_370") { 0x63BBD850 }
        else { 0xE926D3FB }
    )]
    pub safe fn sceAudioInputInitEx(options: &AudioInputParams) -> SceResult<()>;

    /// Performs audio input asynchronously.
    ///
    /// # Parameters
    ///
    /// - `sample_count`: The number of input samples.
    /// - `frequency`: The audio input frequency.
    /// - `buf` **[[Out parameter]]**: The buffer to store the audio input data.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x76BBA42E }
        else if cfg!(feature = "psp_630") { 0x84FA80C4 }
        else if cfg!(feature = "psp_600") { 0xEC8B87CD }
        else if cfg!(feature = "psp_570") { 0x4440159E }
        else if cfg!(feature = "psp_500") { 0x49DA05AF }
        else if cfg!(feature = "psp_420") { 0x8E573FC5 }
        else if cfg!(feature = "psp_395") { 0xF9426C14 }
        else if cfg!(feature = "psp_380") { 0x0B74AAA2 }
        else if cfg!(feature = "psp_370") { 0x3D4B75F7 }
        else { 0x6D4BEC68 }
    )]
    pub unsafe fn sceAudioInput(
        sample_count: u32, frequency: AudioInputFrequency, buf: *mut u8,
    ) -> SceResult<()>;

    /// Performs audio input (blocking).
    ///
    /// That means this functions returns immediately with an error if not able to get the input
    /// audio.
    ///
    /// # Parameters
    ///
    /// - `sample_count`: The number of input samples.
    /// - `frequency`: The audio input frequency.
    /// - `buf` **[[Out parameter]]**: The buffer to store the audio input data.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x42CC2D83 }
        else if cfg!(feature = "psp_630") { 0x459D3B55 }
        else if cfg!(feature = "psp_600") { 0x7F93FE1C }
        else if cfg!(feature = "psp_570") { 0x47160DBA }
        else if cfg!(feature = "psp_500") { 0xE9254026 }
        else if cfg!(feature = "psp_420") { 0xFC314534 }
        else if cfg!(feature = "psp_395") { 0xA8F8D96F }
        else if cfg!(feature = "psp_380") { 0x837D3DF8 }
        else if cfg!(feature = "psp_370") { 0x8BCD60A5 }
        else { 0x086E5895 }
    )]
    pub unsafe fn sceAudioInputBlocking(
        sample_count: u32, frequency: AudioInputFrequency, buf: *mut u8,
    ) -> SceResult<()>;

    /// Gets the number of samples that were acquired.
    ///
    /// # Return Value
    ///
    /// Returns the number of samples acquired on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xFA05EF51 }
        else if cfg!(feature = "psp_630") { 0xE071AB41 }
        else if cfg!(feature = "psp_600") { 0xFA700131 }
        else if cfg!(feature = "psp_570") { 0xD353E4E8 }
        else if cfg!(feature = "psp_500") { 0x83ABA7A8 }
        else if cfg!(feature = "psp_420") { 0x0C6E0800 }
        else if cfg!(feature = "psp_395") { 0x27EBE3E3 }
        else if cfg!(feature = "psp_380") { 0xEE170A13 }
        else if cfg!(feature = "psp_370") { 0x7222BC76 }
        else { 0xA708C6A6 }
    )]
    pub safe fn sceAudioGetInputLength() -> SceResult<u32>;

    /// Waits for a non-blocking audio input to complete.
    ///
    /// # Return Value
    ///
    /// Returns the number of samples acquired on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xD2F5A41A }
        else if cfg!(feature = "psp_630") { 0x8FB1537B }
        else if cfg!(feature = "psp_600") { 0x6B9C59FE }
        else if cfg!(feature = "psp_570") { 0xF882B0E9 }
        else if cfg!(feature = "psp_500") { 0xD364E583 }
        else if cfg!(feature = "psp_420") { 0x2986A38D }
        else if cfg!(feature = "psp_395") { 0x93C9D895 }
        else if cfg!(feature = "psp_380") { 0x3E087113 }
        else if cfg!(feature = "psp_370") { 0x1A223275 }
        else { 0x87B2E651 }
    )]
    pub safe fn sceAudioWaitInputEnd() -> SceResult<()>;

    /// Polls the completion status for non-blocking audio input.
    ///
    /// # Return Value
    ///
    /// Returns the audio input completion status on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[psp_fw_cfg(150..)]
    #[nid(if cfg!(feature = "psp_660") { 0x60B750B2 }
        else if cfg!(feature = "psp_630") { 0x83C00002 }
        else if cfg!(feature = "psp_600") { 0x3076A373 }
        else if cfg!(feature = "psp_570") { 0x01F64F89 }
        else if cfg!(feature = "psp_500") { 0x1408111C }
        else if cfg!(feature = "psp_420") { 0xA9170B12 }
        else if cfg!(feature = "psp_395") { 0x98C058AE }
        else if cfg!(feature = "psp_380") { 0x6ACDCBB3 }
        else if cfg!(feature = "psp_370") { 0xF42AAEFB }
        else { 0xA633048E }
    )]
    pub safe fn sceAudioPollInputEnd() -> SceResult<AudioInputCompletionStatus>;
}

/// Make the given sample count a multiple of 64.
pub const fn audio_sample_align(sample_count: i32) -> i32 {
    (sample_count + 63) & !63
}

impl AudioChannelId {
    /// Channel 0.
    pub const CHANNEL_0: AudioChannelId = unsafe { Self::from_raw_unchecked(0) };
    /// Channel 1.
    pub const CHANNEL_1: AudioChannelId = unsafe { Self::from_raw_unchecked(1) };
    /// Channel 2.
    pub const CHANNEL_2: AudioChannelId = unsafe { Self::from_raw_unchecked(2) };
    /// Channel 3.
    pub const CHANNEL_3: AudioChannelId = unsafe { Self::from_raw_unchecked(3) };
    /// Channel 4.
    pub const CHANNEL_4: AudioChannelId = unsafe { Self::from_raw_unchecked(4) };
    /// Channel 5.
    pub const CHANNEL_5: AudioChannelId = unsafe { Self::from_raw_unchecked(5) };
    /// Channel 6.
    pub const CHANNEL_6: AudioChannelId = unsafe { Self::from_raw_unchecked(6) };
    /// Channel 7.
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
    pub const fn to_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for AudioChannelId {}
unsafe impl SceResultOk for AudioChannelId {
    unsafe fn handle_ok_value(ok_value: u32) -> Option<Self> {
        // On result Channel is never 0xFFFFFFFF
        match ok_value {
            0..=7 => Some(unsafe { Self::from_raw_unchecked(ok_value) }),
            _ => None,
        }
    }
}
unsafe impl SceIntoOkValue for AudioChannelId {
    fn into_ok_value(self) -> u32 {
        self.to_inner()
    }
}

impl crate::private::Sealed for AudioInputCompletionStatus {}
unsafe impl SceResultOk for AudioInputCompletionStatus {
    unsafe fn handle_ok_value(ok_value: u32) -> Option<Self> {
        // On result Channel is never 0xFFFFFFFF
        match ok_value {
            0 => Some(Self::Complete),
            _ => None,
        }
    }
}
unsafe impl SceIntoOkValue for AudioInputCompletionStatus {
    fn into_ok_value(self) -> u32 {
        self as u8 as u32
    }
}

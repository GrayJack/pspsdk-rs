//! Codec operations.

use core::ffi::c_void;

use pspsdk_macros::psp_stub;

use crate::sys::{SceResult, SceSize};

/// Audio codec state and parameter structure.
#[repr(C)]
#[derive(Debug)]
#[doc(alias("SceAudiocodecCodec"))]
pub struct CodecInfo {
    unk0: i32,
    unk1: i32,
    /// Error value if something gone wrong on coded decoding.
    error: i32,
    /// The EDRAM address used by the codec.
    edram_addr: *mut c_void,
    /// The amount of EDRAM required by the codec, in bytes.
    needed_mem: u32,
    unk2: i32,
    /// The input audio buffer.
    in_buffer: *mut u8,
    /// The number of samples read from the input.
    read_sample: u32,
    /// The output decoded audio buffer.
    out_buffer: *mut u8,
    /// The number of samples produced by decoding.
    decoded_sample: u32,

    // Note: this part is probably completely different depending on the codec. This should be
    // cleaned up.
    /// Proabably samplerate
    unk3: i32,
    unk4: i32,
    unk5: i32,
    unk6: i32,
    unk7: i32,
    unk8: i32,
    unk9: i32,
    unk10: i32,
    unk11: i32,
    unk12: i32,
    unk13: i32,
    unk14: i32,
    unk15: i32,
    unk16: i32,
    unk17: i32,
    /// The original allocation returned by `sceMeMalloc`.
    ///
    /// This may differ from `edram_addr` because `edram_addr` is aligned forward to a 64-byte
    /// boundary. The original pointer must be retained so it can be passed to `sceMeFree`.
    alloc_mem: *mut u8,
}

/// The possible codec kinds.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum CodecKind {
    Atrac3Plus = 0x1000,
    Atrac3 = 0x1001,
    Mp3    = 0x1002,
    Aac    = 0x1003,
}

#[psp_stub(libname = "sceAudiocodec", flags = 0x4009, use_crate)]
extern "C" {
    /// Initializes an audio codec instance.
    ///
    /// # Parameters
    ///
    /// - `info` **[[InOut parameter]]**: A pointer to the codec information to be initialized.
    /// - `codec`: The codec kind for this instance.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x5B37EB1D)]
    pub unsafe fn sceAudiocodecInit(info: *mut CodecInfo, codec: CodecKind) -> SceResult<()>;

    /// Checks the memory requirements for an audio codec.
    ///
    /// # Parameters
    ///
    /// - `info` **[[InOut parameter]]**: A pointer to a initialized codec information.
    /// - `codec`: The codec kind for this instance.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x9D3F790C)]
    pub unsafe fn sceAudiocodecCheckNeedMem(
        info: *mut CodecInfo, codec: CodecKind,
    ) -> SceResult<()>;

    /// Decoded an audio buffer.
    ///
    /// # Parameters
    ///
    /// - `info` **[[InOut parameter]]**: A pointer to a initialized codec information. The input
    ///   and output buffer must be set.
    /// - `codec`: The codec kind for this instance.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x70A703F8)]
    pub unsafe fn sceAudiocodecDecode(info: *mut CodecInfo, codec: CodecKind) -> SceResult<()>;

    /// Gets/updates a codec specific information.
    ///
    /// # Parameters
    ///
    /// - `info` **[[InOut parameter]]**: A pointer to a initialized codec information.
    /// - `codec`: The codec kind for this instance.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x8ACA11D5)]
    pub unsafe fn sceAudiocodecGetInfo(info: *mut CodecInfo, codec: CodecKind) -> SceResult<()>;

    /// Allocates and assigns EDRAM for a codec instance.
    ///
    /// # Parameters
    ///
    /// - `info` **[[InOut parameter]]**: A pointer to a initialized codec information.
    /// - `codec`: The codec kind for this instance.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x3A20A200)]
    pub unsafe fn sceAudiocodecGetEDRAM(info: *mut CodecInfo, codec: CodecKind) -> SceResult<()>;

    /// Releases EDRAM previously allocated for a codec instance.
    ///
    /// # Parameters
    ///
    /// - `info` **[[InOut parameter]]**: A pointer to a initialized codec information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x29681260)]
    pub unsafe fn sceAudiocodecReleaseEDRAM(info: *mut CodecInfo) -> SceResult<()>;

    /// Calculates the codec's extended output parameter size.
    ///
    /// # Parameters
    ///
    /// - `info` **[[InOut parameter]]**: A pointer to a initialized codec information.
    /// - `codec`: The codec kind for this instance.
    /// - `size` **[[Out parameter]]**: A reference to receive  the calculated size in bytes.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x59176A0F)]
    pub unsafe fn sceAudiocodecAlcExtendParameter(
        info: *mut CodecInfo, codec: CodecKind, size: &mut SceSize,
    ) -> SceResult<()>;
}

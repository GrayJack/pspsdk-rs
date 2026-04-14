#![allow(unused_imports)]

use core::ffi::c_void;

use pspsdk_macros::psp_stub;

use crate::sys::{SceError, SceResult, SceResultOk, SceSize, SceUid};

/// Identification of Atrac3 objects.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AtracId(SceUid);

#[repr(C)]
#[derive(Debug, Clone)]
#[doc(alias("SceBufferInfo", "PspBufferInfo"))]
pub struct AtracBufferInfo {
    pub write_position_first_buf: *mut u8,
    pub writable_byte_first_buf: SceSize,
    pub min_write_byte_first_buf: SceSize,
    pub read_position_first_buf: SceSize,
    pub write_position_second_buf: *mut u8,
    pub writable_byte_second_buf: SceSize,
    pub min_write_byte_second_buf: SceSize,
    pub read_position_second_buf: SceSize,
}

/// The looping status of the Atrac object.
///
/// This information than be received from [`sceAtracGetLoopStatus`].
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias = "SceAtracLoopStatus")]
pub enum AtracLoopStatus {
    /// The Atrac object is not set to loop or currently looping.
    NotLooping = 0x0,
    /// The Atrac object is set to loop or currently looping.
    Looping = 0x1,
}

/// The possible kinds of ATRAC codec.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AtracCodecKind {
    #[doc(alias("SCE_ATRAC_AT3", "PSP_ATRAC_AT3"))]
    Atrac3 = 0x00001001,
    #[doc(alias("SCE_ATRAC_AT3PLUS", "PSP_ATRAC_AT3PLUS"))]
    Atrac3Plus = 0x00001000,
}

// FIXME: Add missing docs and missing function
#[psp_stub(libname = "sceAtrac3plus", flags = 0x0009, use_crate)]
extern "C" {
    /// Get the Atrac ID for an available/released Atrac object with the specified `codec_kind`.
    ///
    /// # Parameters
    /// - `codec_kind`: The kind of codec requested.
    ///
    /// # Return Value
    ///
    /// Returns an Atrac ID on success, an error value otherwise.
    #[nid(0x780F88D1)]
    pub fn sceAtracGetAtracID(codec_kind: AtracCodecKind) -> SceResult<AtracId>;

    /// Creates a new Atrac ID from the specified data.
    ///
    /// # Parameters
    ///
    /// - `buf`: The buffer holding the Atrac3 data, including the RIFF/WAVE header.
    /// - `bufsize`: The size of the buffer pointed by `buf`.
    ///
    /// # Return Value
    ///
    /// Returns the new Atrac ID on success, an error value otherwise.
    #[nid(0x7A20E7AF)]
    pub unsafe fn sceAtracSetDataAndGetID(buf: *mut c_void, bufsize: usize) -> SceResult<AtracId>;

    /// Decode a frame of data.
    ///
    /// # Parameters
    ///
    /// - `atrac_id`: The atrac ID.
    /// - `samples` **[[Out parameter]]**: A pointer to a buffer that receives the decoded data of
    ///   the current frame.
    /// - `num_samples` **[[Out parameter]]**: A reference to a integer that receives the number of
    ///   audio samples of the decoded frame.
    /// - `end` **[[Out parameter]]**: A reference to a integer that receives a boolean value
    ///   indicating if the decoded frame is the last one.
    /// - `remain_frame` **[[Out parameter]]**: A reference to a integer that receives either `-1``
    ///   if all atrac3 data is already on memory, or the remaining (not decoded yet) frames at
    ///   memory if not all atrac3 data is on memory.
    ///
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[eabi(i5)]
    #[nid(0x6A8C3CD5)]
    pub unsafe fn sceAtracDecodeData(
        atrac_id: AtracId, samples: *mut u16, num_samples: &mut i32, end: &mut i32,
        remain_frame: &mut i32,
    ) -> SceResult<()>;

    /// Gets the remaining (not decoded) number of frames
    ///
    /// # Parameters
    ///
    /// - `atrac_id`: The Atrac ID.
    /// - `remain_frame` **[[Out parameter]]**: A reference to a integer that receives either `-1``
    ///   if all atrac3 data is already on memory, or the remaining (not decoded yet) frames at
    ///   memory if not all atrac3 data is on memory.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x9AE849A7)]
    pub fn sceAtracGetRemainFrame(atrac_id: AtracId, remain_frame: &mut i32) -> SceResult<()>;

    /// Get information of the Atrac stream data of the given Atrac ID.
    ///
    /// # Parameters
    ///
    /// - `atrac_id`: The Atrac ID.
    /// - `write_pointer` **[[Out parameter]]**: A reference to where to read the atrac data.
    /// - `available_bytes` **[[Out parameter]]**: Number of bytes available at the `write_pointer`
    ///   location.
    /// - `read_offset` **[[Out parameter]]**: Offset where to seek into the atrac file before
    ///   reading.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x5D268707)]
    pub fn sceAtracGetStreamDataInfo(
        atrac_id: AtracId, write_pointer: &mut *mut u8, available_bytes: &mut SceSize,
        read_offset: &mut SceSize,
    ) -> SceResult<()>;

    /// Adds a specified number of bytes to the atrac stream data.
    ///
    /// # Parameters
    ///
    /// - `atrac_id`: The Atrac ID.
    /// - `bytes_to_add`: Number of bytes to read into location given by
    ///   [`sceAtracGetStreamDataInfo`].
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x7DB31251)]
    pub fn sceAtracAddStreamData(atrac_id: AtracId, bytes_to_add: SceSize) -> SceResult<()>;

    /// Gets the Atrac object bitrate.
    ///
    /// # Parameters
    ///
    /// - `atrac_id`: The Atrac ID.
    /// - `bitrate` **[[Out parameter]]**: A reference to a integer that receives the bitrate in
    ///   _kbps_.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xA554A158)]
    pub fn sceAtracGetBitrate(atrac_id: AtracId, bitrate: &mut i32) -> SceResult<()>;

    /// Sets the number of loops for the Atrac object associated to the given Atrac ID.
    ///
    /// # Parameters
    ///
    /// - `atrac_id`: The Atrac ID.
    /// - `nloops`: The number of loops to set.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x868120B5)]
    pub fn sceAtracSetLoopNum(atrac_id: AtracId, num_loops: i32) -> SceResult<()>;

    /// Releases an Atrac ID
    ///
    /// # Parameters
    ///
    /// - `atrac_id`: The Atrac ID.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x61EB33F5)]
    pub fn sceAtracReleaseAtracID(atrac_id: AtracId) -> SceResult<()>;

    /// Gets the number of samples of the next frame to be decoded.
    ///
    /// # Parameters
    ///
    /// - `atrac_id`: The Atrac ID.
    /// - `num_samples` **[[Out parameter]]**: A reference to receives the number of samples of the
    ///   next frame.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x36FAABFB)]
    pub fn sceAtracGetNextSample(atrac_id: AtracId, num_samples: &mut i32) -> SceResult<()>;

    /// Gets the maximum number of samples of the Atrac3 stream.
    ///
    /// # Parameters
    ///
    /// - `atrac_id`: The Atrac ID.
    /// - `max_samples` **[[Out parameter]]**: A reference to a integer that receives the maximum
    ///   number of samples.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xD6A5F2F7)]
    pub fn sceAtracGetMaxSample(atrac_id: AtracId, max_samples: &mut i32) -> SceResult<()>;

    /// Gets the buffer information to reset the Atrac buffer.
    ///
    /// # Parameters
    ///
    /// - `atrac_id`: The Atrac ID.
    /// - `sample`: The sample to get the buffer information.
    /// - `buffer_info` **[[Out parameter]]**: A reference to a [`AtracBufferInfo`] structure to
    ///   receive the buffer information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xCA3CA3D2)]
    pub fn sceAtracGetBufferInfoForReseting(
        atrac_id: AtracId, sample: u32, buffer_info: &mut AtracBufferInfo,
    ) -> SceResult<()>;

    /// Get the channel of a Atrac object.
    ///
    /// # Parameters
    ///
    /// - `atrac_id`: The Atrac ID.
    /// - `channel` **[[Out parameter]]**: A reference to a integer to receive the channel number.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x31668BAA)]
    pub fn sceAtracGetChannel(atrac_id: AtracId, channel: &mut u32) -> SceResult<()>;

    /// Get the internal Codec error from the Atrac object.
    ///
    /// # Parameters
    ///
    /// - `atrac_id`: The Atrac ID.
    /// - `codec_error` **[[Out parameter]]**: A reference to a possible codec error.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xE88F759B)]
    pub fn sceAtracGetInternalErrorInfo(
        atrac_id: AtracId, codec_error: &mut Option<SceError>,
    ) -> SceResult<()>;


    /// Get the loop status and loop iteration from the Atrac object.
    ///
    /// # Parameters
    ///
    /// - `atrac_id`: The Atrac ID.
    /// - `loop_num` **[[Out parameter]]**: A reference to a integer to receive the loop counter.
    /// - `loop_status` **[[Out parameter]]**: A reference to a [`AtracLoopStatus`] ti receive the
    ///   loop status information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xFAA4F89B)]
    pub fn sceAtracGetLoopStatus(
        atrac_id: AtracId, loop_num: &mut SceSize, loop_status: &mut AtracLoopStatus,
    ) -> SceResult<()>;

    /// Gets the next decode sample position.
    ///
    /// # Parameters
    ///
    /// - `atrac_id`: The Atrac ID.
    /// - `sample_position` **[[Out parameter]]**: A reference to a integer to receive the next
    ///   sample position.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xE23E3A35)]
    pub fn sceAtracGetNextDecodePosition(
        atrac_id: AtracId, sample_position: &mut SceSize,
    ) -> SceResult<()>;

    /// Get the second buffer information of the Atrac object.
    ///
    /// # Parameters
    ///
    /// - `atrac_id`: The Atrac ID.
    /// - `position` **[[Out parameter]]**: A reference to a integer to receive the second buffer
    ///   position.
    /// - `data_byte` **[[Out parameter]]**: A reference to a integer to receive the second buffer
    ///   data byte.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x83E85EA0)]
    pub fn sceAtracGetSecondBufferInfo(
        atrac_id: AtracId, position: &mut SceSize, data_byte: &mut u32,
    ) -> SceResult<()>;

    /// Get the sound sample information of the Atrac object.
    ///
    /// # Parameters
    ///
    /// - `atrac_id`: The Atrac ID.
    /// - `end_sample` **[[Out parameter]]**: A reference to a integer to receive the end
    ///   information of the sample.
    /// - `loop_start_sample` **[[Out parameter]]**: A reference to a integer to receive the loop
    ///   start information of the sample. It receiver `-1` when sample not looping/set to loop.
    /// - `loop_end_sample` **[[Out parameter]]**: A reference to a integer to receive the loop end
    ///   information of the sample. It receiver `-1` when sample not looping/set to loop.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xA2BBA8BE)]
    pub fn sceAtracGetSoundSample(
        atrac_id: AtracId, end_sample: &mut i32, loop_start_sample: &mut i32,
        loop_end_sample: &mut i32,
    ) -> SceResult<()>;

    /// Resets the play position of the sample of a Atrac object.
    ///
    /// # Parameters
    ///
    /// - `atrac_id`: The Atrac ID.
    /// - `sample`: The sample to reset the play position.
    /// - `write_offset_first_buf`: The write offset of the first buffer to reset the play position.
    /// - `write_offset_second_buf`: The write offset of the second buffer to reset the play
    ///   position position.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x644E5607)]
    pub fn sceAtracResetPlayPosition(
        atrac_id: AtracId, sample: u32, write_offset_first_buf: SceSize,
        write_offset_second_buf: SceSize,
    ) -> SceResult<()>;

    #[nid(0x0E2A73AB)]
    pub unsafe fn sceAtracSetData(
        atrac_id: AtracId, buffer_addr: *mut u8, buffer_byte: SceSize,
    ) -> SceResult<()>;

    #[nid(0x3F6E26B5)]
    pub unsafe fn sceAtracSetHalfwayBuffer(
        atrac_id: AtracId, buffer_addr: *mut u8, read_byte: u32, buffer_byte: u32,
    ) -> SceResult<()>;

    #[nid(0x0FAE370E)]
    pub unsafe fn sceAtracSetHalfwayBufferAndGetID(
        buffer_addr: *mut u8, read_byte: u32, buffer_byte: u32,
    ) -> SceResult<AtracId>;

    #[nid(0x83BF7AFD)]
    pub unsafe fn sceAtracSetSecondBuffer(
        atrac_id: AtracId, second_buffer_addr: *mut u8, second_buffer_byte: u32,
    ) -> SceResult<()>;
}


impl AtracId {
    /// Create a new Atrac ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceAtracId`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        if let 0..6 = raw {
            Some(unsafe { Self::from_raw_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new Atrac ID structure from a raw value without checking value range.
    ///
    /// # Safety
    ///
    /// Immediate language UB if `val` is not within the valid range for this
    /// type, as it violates the validity invariant.
    #[inline]
    pub const unsafe fn from_raw_unchecked(raw: u32) -> Self {
        Self(unsafe { SceUid::from_raw_unchecked(raw) })
    }

    #[inline]
    pub const fn as_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for AtracId {}
unsafe impl SceResultOk for AtracId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        match ok_value {
            0..6 => Ok(unsafe { Self::from_raw_unchecked(ok_value) }),
            _ => Err(SceError::INVALID_VALUE),
        }
    }
}

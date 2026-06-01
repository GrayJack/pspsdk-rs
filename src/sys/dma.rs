//! Direct Memory Access (DMA) functions.
#[cfg(any(target_os = "psp", doc))]
use core::ffi::c_void;

use pspsdk_macros::psp_stub;

use crate::sys::{SceResult, SceSize};


#[psp_stub(libname = "sceDmac", flags = 0x4001, use_crate)]
extern "C" {
    /// Perform a DMA memory copy (blocking).
    ///
    /// Copies `size` bytes from `src` to `dst` using the DMA controller.
    ///
    /// This call blocks until the transfer is complete.
    ///
    /// # Parameters
    ///
    /// - `dst` **[[Out parameter]]**: A pointer to the destination memory.
    /// - `src` **[[In parameter]]**: A pointer to the source memory.
    /// - `size`: The number of bytes to copy.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x617F3FE6)]
    pub unsafe fn sceDmacMemcpy(
        dst: *mut c_void, src: *const c_void, size: SceSize,
    ) -> SceResult<()>;

    /// Perform a DMA memory copy (non-blocking attempt).
    ///
    /// Like `sceDmacMemcpy`, but returns immediately with an error if
    /// the DMA controller is busy.
    ///
    /// # Parameters
    ///
    /// - `dst` **[[Out parameter]]**: A pointer to the destination memory.
    /// - `src` **[[In parameter]]**: A pointer to the source memory.
    /// - `size`: The number of bytes to copy.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise (including busy).
    #[nid(0xD97F94D8)]
    pub unsafe fn sceDmacTryMemcpy(
        dst: *mut c_void, src: *const c_void, size: SceSize,
    ) -> SceResult<()>;
}

// FIXME: Add `sceDmacplus_driver` functions (not required for PSPSDK parity)

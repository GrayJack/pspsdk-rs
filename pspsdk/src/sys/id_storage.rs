//! ID Storage API.

use core::ffi::c_void;

use pspsdk_macros::psp_stub;

use crate::sys::{SceResult, SceSize};

/// A callback that is called by [`sceIdStorageEnumId`] for each ID Storage leaf.
///
/// # Parameters
///
/// - `leaf_id`: The leaf identifier.
/// - `ppn`: The physical page number containing the leaf.
/// - `common` **[[InOut parameter]]**: A pointer to memory shared with the callback.
#[doc(alias("sceIdStorageEnumCB"))]
pub type EnumCallback = unsafe extern "C" fn(leaf_id: u16, ppn: i32, common: *mut c_void);


#[psp_stub(libname = "sceIdStorage_driver", flags = 0x0009, use_crate)]
extern "C" {
    /// Gets the size of the ID storage leaf.
    ///
    /// # Return Value
    ///
    /// Returns the leaf size in bytes.
    #[nid(0xEB830733)]
    pub safe fn sceIdStorageGetLeafSize() -> SceSize;

    /// Checks if the ID Storage is formatted.
    #[nid(0xFEFA40C2)]
    pub safe fn sceIdStorageIsFormatted() -> bool;

    /// Checks if the ID Storage is in read-only state.
    #[nid(0x2D633688)]
    pub safe fn sceIdStorageIsReadOnly() -> bool;

    /// Checks if the ID Storage contains unflushed changes.
    #[nid(0xB9069BAD)]
    pub safe fn sceIdStorageIsDirty() -> bool;

    /// Formats the ID Storage area, erasing existing leaves.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x958089DB)]
    pub unsafe fn sceIdStorageFormat() -> SceResult<()>;

    /// Unformats the ID storage area by erasing its blocks.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xF4BCB3EE)]
    pub unsafe fn sceIdStorageUnformat() -> SceResult<()>;

    /// Reads the leaf container associated to a key.
    ///
    /// # Parameters
    ///
    /// - `leaf_id`: The leaf identifier.
    /// - `buf` **[[Out parameter]]**: A buffer to receive the leaf container. It must valid to
    ///   write for `512` bytes.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xEB00C509)]
    pub unsafe fn sceIdStorageReadLeaf(leaf_id: u16, buf: *mut u8) -> SceResult<()>;

    /// Writes a complete ID storage leaf.
    ///
    /// # Parameters
    ///
    /// - `leaf_id`: The leaf identifier.
    /// - `buf` **[[In parameter]]**: A buffer of a leaf container. It must valid to read for `512`
    ///   bytes.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x1FA4D135)]
    pub unsafe fn sceIdStorageWriteLeaf(leaf_id: u16, buf: *const u8) -> SceResult<()>;

    /// Flushes pending leaf and index changes to NAND.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x3AD32523)]
    pub safe fn sceIdStorageFlush() -> SceResult<()>;

    /// Gets the number of unused leaf slots.
    ///
    /// # Return Value
    ///
    /// Returns the number of unused leaf slots on success, error value otherwise.
    #[nid(0x37833CB8)]
    pub safe fn sceIdStorageGetFreeLeaves() -> SceResult<u32>;

    /// Executes a callback for every existing ID Storage leaves.
    ///
    /// # Parameters
    ///
    /// - `callback` **[[In parameter]]**: The callback to be executed.
    /// - `commmon` **[[InOut parameter]]**: A user-defined value/structure passed to the callback
    ///   on every call.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x31E08AFB)]
    pub unsafe fn sceIdStorageEnumId(callback: EnumCallback, common: *mut c_void) -> SceResult<()>;

    /// Creates an empty ID Storage leaf.
    ///
    /// # Parameters
    ///
    /// - `leaf_id`: The leaf identifier to create.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x08A471A6)]
    pub unsafe fn sceIdStorageCreateLeaf(leaf_id: u16) -> SceResult<()>;

    /// Atomically creates multiple empty ID Storage leaves.
    ///
    /// # Parameters
    ///
    /// - `leaf_ids` **[[In parameter]]**: A array pointer with the identifiers to create.
    /// - `size`: The size of the `leaf_ids`.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x99ACCB71)]
    pub unsafe fn sceIdStorageCreateAtomicLeaves(
        leaf_ids: *const u16, size: SceSize,
    ) -> SceResult<()>;

    /// Deletes an ID storage leaf.
    ///
    /// # Parameters
    ///
    /// - `leaf_id`: The leaf identifier to delete.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x2C97AB36)]
    pub unsafe fn sceIdStorageDeleteLeaf(leaf_id: u16) -> SceResult<()>;

    /// Reads arbitrary amount of an ID Storage leaf.
    ///
    /// # Parameters
    ///
    /// - `leaf_id`: The leaf identifier.
    /// - `offset`: The byte offset withing the 512 bytes leaf.
    /// - `buf` **[[Out parameter]]**: A buffer to receive the leaf data.
    /// - `size`: The number of bytes to read.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x6FE062D1)]
    pub unsafe fn sceIdStorageLookup(
        leaf_id: u16, offset: SceSize, buf: *mut u8, size: SceSize,
    ) -> SceResult<()>;

    /// Updates arbitrary amount of an ID Storage leaf.
    ///
    /// Changes are written to NAND when [`sceIdStorageFlush`] is called.
    ///
    /// # Parameters
    ///
    /// - `leaf_id`: The leaf identifier.
    /// - `offset`: The byte offset withing the 512 bytes leaf.
    /// - `buf` **[[In parameter]]**: A buffer with the leaf data to be updated.
    /// - `size`: The number of bytes to write.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x683AAC10)]
    pub unsafe fn sceIdStorageUpdate(
        leaf_id: u16, offset: SceSize, buf: *const u8, size: SceSize,
    ) -> SceResult<()>;
}

//! Memory management

use core::ffi::{c_char, c_void};

use pspsdk_macros::psp_stub;

use crate::sys::{SceError, SceIsize, SceResult, SceResultOk, SceSize, SceUid};

/// The memory block UID.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemoryBlockId(SceUid);

/// The RAM partition ID
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum MemoryPartitionId {
    /// Unknown. Sometimes used to default to [`MainKernel`](MemoryPartitionId::MainKernel) if
    /// kernel call and [`MainUser`](MemoryPartitionId::MainUser) if user call.
    #[default]
    Unknown = 0,
    /// Principal kernel partition. Usually 8MB.
    MainKernel = 1,
    /// Principal user partition. Usually 24MB.
    MainUser = 2,
    /// Other kernel partition.
    OtherKernel = 3,
    /// Other kernel partition.
    OtherKernel2 = 4,
    /// Visual Shell partition.
    Vshell = 5,
    /// Syscon user partition.
    SysconUser = 6,
    /// Media Engine user partition.
    MeUser = 7,
    /// Extended syscon kernel partition.
    ExtendedSysconKernel = 8,
    /// Extended syscon kernel partition.
    ExtendedSysconKernel2 = 9,
    /// Media Engine kernel partition.
    ExtendedMeKernel = 10,
    /// Visual Shell kernel partition.
    ExtendedVshell = 11,
    /// Extended kernel partition.
    ExtendedKernel = 12,
}

/// Type the specifies the kind of allocation is used for memory blocks.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum MemoryBlockKind {
    /// Allocate from the lowest available address.
    #[default]
    Low  = 0,
    /// Allocate from the highest available address.
    High = 1,
    /// Allocate from the specified address.
    Addr = 2,
    /// Allocate from the lowest available address aligned to the specified address.
    LowAligned = 3,
    /// Allocate from the highest available address aligned to the specified address.
    HighAligned = 4,
}

/// Information of a memory partition.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemoryPartitionInfo {
    /// The partition initial RAM address.
    pub addr: SceSize,
    /// The partition size in bytes.
    pub size: SceSize,
}


/// Snapshot of the system memory partition layout.
///
/// This table is typically used by PSP kernel internal APIs.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemoryPartitionTable {
    /// Total memory size (?).
    pub mem_size: SceSize,
    pub unk0: u32,
    pub unk1: u32,
    /// [`MemoryPartitionId::OtherKernel`] partition information.
    pub other1: MemoryPartitionInfo,
    /// [`MemoryPartitionId::OtherKernel2`] partition information.
    pub other2: MemoryPartitionInfo,
    /// [`MemoryPartitionId::Vshell`] partition information.
    pub vshell: MemoryPartitionInfo,
    /// [`MemoryPartitionId::SysconUser`] partition information.
    pub sc_user: MemoryPartitionInfo,
    /// [`MemoryPartitionId::MeUser`] partition information.
    pub me_user: MemoryPartitionInfo,
    /// [`MemoryPartitionId::ExtendedSysconKernel2`] partition information.
    pub ext_sc_kernel2: MemoryPartitionInfo,
    /// [`MemoryPartitionId::ExtendedSysconKernel`] partition information.
    pub ext_sc_kernel1: MemoryPartitionInfo,
    /// [`MemoryPartitionId::ExtendedMeKernel`] partition information.
    pub ext_me_kernel: MemoryPartitionInfo,
    /// [`MemoryPartitionId::ExtendedVshell`] partition information.
    pub ext_vshell: MemoryPartitionInfo,
}

/// Extra options to pass to [`sceKernelAllocMemoryBlock`].
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemoryBlockAllocOptions {
    size: SceSize,
}

#[psp_stub(libname = "SysMemUserForUser", flags = 0x4000)]
extern "C" {
    /// Allocate a memory block from a memory partition.
    ///
    /// # Parameters
    ///
    /// - `partition`: The partition ID for the partition to allocate from.
    /// - `name`: Name assigned to the new block. Only used for debug.
    /// - `kind`: Specifies how the block is allocated within the partition.
    /// - `size`: Size of the memory block, in bytes.
    /// - `addr`: If `kind` is [`MemoryBlockKind::Addr`], then addr specifies the lowest address
    ///   allocate the block from.
    ///
    /// # Return Value
    ///
    /// Returns an UID of the new block on success, an error value otherwise.
    #[eabi(i5)]
    #[nid(0x237DBD4F)]
    pub unsafe fn sceKernelAllocPartitionMemory(
        partition: MemoryPartitionId, name: *const u8, kind: MemoryBlockKind, size: SceSize,
        addr: SceSize,
    ) -> SceResult<MemoryBlockId>;

    /// Get the address of a memory block.
    ///
    /// # Parameters
    ///
    /// `block_id`: The UID of the memory block.
    ///
    /// # Return Values
    ///
    /// Returns the lowest address belonging to the memory block.
    #[nid(0x9D9A5BA1)]
    pub fn sceKernelGetBlockHeadAddr(block_id: MemoryBlockId) -> *mut c_void;

    /// Free a memory block allocated with [`sceKernelAllocPartitionMemory`].
    ///
    /// # Parameters
    ///
    /// - `block_id`: The UID of the block to free.
    ///
    /// # Return value
    ///
    /// Returns an unknown value on success, an error value otherwise.
    #[nid(0xB6D61D02)]
    pub unsafe fn sceKernelFreePartitionMemory(block_id: MemoryBlockId) -> SceResult<u32>;

    /// Allocate a memory block from the main user partition.
    ///
    /// # Parameters
    ///
    /// - `name`: Name assigned to the new block. Only used for debug.
    /// - `kind`: Specifies how the block is allocated within the partition.
    /// - `size`: Size of the memory block, in bytes.
    ///
    /// # Return Value
    ///
    /// Returns an UID of the new block on success, an error value otherwise.
    #[nid(0xFE707FDF)]
    pub unsafe fn sceKernelAllocMemoryBlock(
        name: *const u8,
        kind: MemoryBlockKind,
        size: SceSize,
        // Original is `*const MemoryBlockAllocOptions`
        // We know the implementation, that Can take NULL or valid ptr (reference) to
        // `MemoryBlockAllocOptions` and only read the options Knowing that, we can safely
        // use `Option<&MemoryBlockAllocOptions>`
        opt: Option<&MemoryBlockAllocOptions>,
    ) -> SceResult<MemoryBlockId>;

    /// Get the address of a memory block.
    ///
    /// # Parameters
    ///
    /// - `block_id`: The UID of the memory block.
    /// - `addr` **[[Out parameter]]**: A reference to receive the address.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0xDB83A952)]
    pub unsafe fn sceKernelGetMemoryBlockAddr(
        block_id: MemoryBlockId, addr: &mut *mut c_void,
    ) -> SceResult<()>;

    /// Free a memory block allocated with [`sceKernelAllocMemoryBlock`].
    ///
    /// # Parameters
    ///
    /// - `block_id`: The UID of the block to free.
    ///
    /// # Return value
    ///
    /// Returns an unknown value on success, an error value otherwise.
    #[nid(0x50F61D8A)]
    pub unsafe fn sceKernelFreeMemoryBlock(block_id: MemoryBlockId) -> SceResult<i32>;

    /// Obtains the memory information of a given address.
    ///
    /// # Parameters
    ///
    /// - `addr`: The memory address to query for information.
    /// - `partition_id` **[[Out parameter]]**: A reference to receive the memory partition ID
    ///   information. Receives `-1` if information not found.
    /// - `block_id` **[[Out parameter]]**: A reference to receive the memory block UID information.
    ///   Receives `-1` if information not found.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x2A3E5280)]
    pub fn sceKernelQueryMemoryInfo(
        addr: SceSize, partition_id: &mut SceIsize, block_id: &mut SceIsize,
    ) -> SceResult<()>;

    /// Get the total size of the main user memory partition.
    ///
    /// # Return Value
    ///
    /// The total size of the main user memory partition in bytes.
    #[nid(0xACBD88CA)]
    pub fn sceKernelTotalMemSize() -> SceSize;

    /// Get the total amount of free memory.
    ///
    /// # Return Value
    ///
    /// The total amount of free memory, in bytes.
    #[nid(0xF919F628)]
    pub fn sceKernelTotalFreeMemSize() -> SceSize;

    /// Get the size of the largest free memory block.
    ///
    /// # Return Value
    ///
    /// The size of the largest free memory block, in bytes.
    #[nid(0xA291F107)]
    pub fn sceKernelMaxFreeMemSize() -> SceSize;

    /// Get the firmware version.
    ///
    /// # Return Value
    ///
    /// The firmware version.
    ///
    /// - `0x01000300` on v1.00 unit,
    /// - `0x01050001` on v1.50 unit,
    /// - `0x01050100` on v1.51 unit,
    /// - `0x01050200` on v1.52 unit,
    /// - `0x02000010` on v2.00/v2.01 unit,
    /// - `0x02050010` on v2.50 unit,
    /// - `0x02060010` on v2.60 unit,
    /// - `0x02070010` on v2.70 unit,
    /// - `0x02070110` on v2.71 unit.
    #[nid(0x3FC9AE6A)]
    pub fn sceKernelDevkitVersion() -> u32;

    /// Set the version of the SDK with which the caller was compiled.
    ///
    /// Version numbers are the same as for [`sceKernelDevkitVersion`].
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x7591C7DB)]
    pub fn sceKernelSetCompiledSdkVersion(version: u32) -> SceResult<()>;

    /// Get the SDK version set with [`sceKernelSetCompiledSdkVersion`].
    ///
    /// # Return Value
    ///
    /// Returns the version number, or `0` if unset.
    #[nid(0xFC114573)]
    pub fn sceKernelGetCompiledSdkVersion() -> u32;

    /// Kernel printf function.
    ///
    /// # Parameters
    ///
    /// - `format`: The format string.
    /// - `...`: Variables to print.
    #[nid(0x13A5ABEF)]
    pub unsafe fn sceKernelPrintf(format: *const c_char, ...);
}

impl MemoryBlockId {
    /// Create a new Atrac ID from a raw value.
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

    /// Create a new Atrac ID structure from a raw value without checking value range.
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

impl crate::private::Sealed for MemoryBlockId {}
unsafe impl SceResultOk for MemoryBlockId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe { SceUid::handle_ok_value(ok_value).map(Self) }
    }
}

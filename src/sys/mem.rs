//! Memory management
#![allow(unused_imports)]

use core::ffi::{c_char, c_void, VaList};
#[cfg(feature = "kernel")]
use core::ptr::NonNull;

use bitflag_attr::bitflag;
use pspsdk_macros::psp_stub;

use crate::sys::{SceError, SceIsize, SceResult, SceResultOk, SceSize, SceUid};

/// The memory block UID.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemoryBlockId(SceUid);

/// The heap UID.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HeapId(SceUid);

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

/// Basic information of a memory partition.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemoryPartitionBasicInfo {
    /// The partition initial RAM address.
    pub addr: SceSize,
    /// The partition size in bytes.
    pub size: SceSize,
}

/// Full information of a memory partition.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemoryPartitionInfo {
    pub size: SceSize,
    pub start_addr: SceSize,
    pub mem_size: SceSize,
    pub attributes: u32,
}

/// Function type for [`sceKernelSetUsersystemLibWork`].
pub type GeListUpdateStallAddrLazy = unsafe extern "C" fn(i32, *mut c_void) -> i32;

/// Struct passed to [`sceKernelSetUsersystemLibWork`].
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GeLazy {
    /// Last display list for which a [`GeListUpdateStallAddrLazy`] was run.
    pub display_list_id: SceIsize,
    // The stall address which was supposed to be set in the last call.
    pub stall: *mut c_void,
    // Number of times an update has been called on the current `display_list_id`.
    pub count: SceSize,
    // Number of calls to [`GeListUpdateStallAddrLazy`] required until we really set the address.
    pub max: SceSize,
}

/// Flags to create and configure heap behavior on [`sceKernelCreateHeap`].
#[bitflag(u32)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum HeapCreateFlag {
    /// Unknown. Default value.
    #[default]
    Unknown = 1 << 0,
    /// Allocate the heap from the lowest available address.
    Low = 1 << 1,
}

/// Allocation options for the [`sceKernelAllocHeapMemoryWithOption`].
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HeapAllocOptions {
    /// The size of this structure... probably
    pub size: SceSize,
    /// The allocation alignment.
    pub align: SceSize,
}

/// Information of a heap object in the system.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HeapInfo {
    /// The size of this structure.
    pub size: SceSize,
    /// The name of the heap object.
    pub name: [u8; 32],
    /// The heap memory permission.
    pub permission: i32,
    /// The heap attributes.
    pub attribute: i32,
    /// The heap current size.
    pub heap_size: SceSize,
    /// The heap total size.
    pub total_size: SceSize,
    /// The heap total free size.
    pub total_free_size: SceSize,
    /// The size of the largest free memory block in the heap.
    pub max_free_size: SceSize,
    /// The number heap blocks in the heap.
    pub num_heaps: SceSize,
    /// The root (?) of the heap block information
    pub heap_blocks: *mut HeapBlock,
}

/// Heap block information.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct HeapBlock {
    pub next: *mut HeapBlock,
    pub previous: *mut HeapBlock,
}


#[repr(C)]
#[derive(Debug, Clone)]
pub struct LowHeapInfo {
    /// The size of this structure.
    pub size: SceSize,
    /// The heap current size.
    pub heap_size: SceSize,
    /// The used size.
    pub used_size: SceSize,
    /// The heap total free size.
    pub total_free_size: SceSize,
    /// The size of the largest free memory block in the heap.
    pub max_free_size: SceSize,
    /// Number of blocks in the `info_block`.
    pub block_count: SceSize,
    /// The info block list.
    pub info_block: *mut LowheapInfoBlock,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct LowheapInfoBlock {
    pub block: *mut LowheapInfoBlock,
    pub offset: SceSize,
}

/// Low Heap block information.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct LowHeapBlock {
    pub next: *mut LowHeapBlock,
    pub count: SceSize,
}

/// PSP Hardware models.
#[repr(u32)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PspHardwareModel {
    /// PSP Fat (01g).
    Psp01g = 0,
    /// PSP Slim (02g).
    Psp02g = 1,
    /// PSP Brite (03g).
    Psp03g = 2,
    /// PSP Brite (04g).
    Psp04g = 3,
    /// PSP Go (05g).
    Psp05g = 4,
    /// PSP Brite (07g).
    Psp07g = 6,
    /// PSP Brite (09g).
    Psp09g = 8,
    /// PSP Street E-1000 (11g).
    Psp11g = 10,
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
    pub other1: MemoryPartitionBasicInfo,
    /// [`MemoryPartitionId::OtherKernel2`] partition information.
    pub other2: MemoryPartitionBasicInfo,
    /// [`MemoryPartitionId::Vshell`] partition information.
    pub vshell: MemoryPartitionBasicInfo,
    /// [`MemoryPartitionId::SysconUser`] partition information.
    pub sc_user: MemoryPartitionBasicInfo,
    /// [`MemoryPartitionId::MeUser`] partition information.
    pub me_user: MemoryPartitionBasicInfo,
    /// [`MemoryPartitionId::ExtendedSysconKernel2`] partition information.
    pub ext_sc_kernel2: MemoryPartitionBasicInfo,
    /// [`MemoryPartitionId::ExtendedSysconKernel`] partition information.
    pub ext_sc_kernel1: MemoryPartitionBasicInfo,
    /// [`MemoryPartitionId::ExtendedMeKernel`] partition information.
    pub ext_me_kernel: MemoryPartitionBasicInfo,
    /// [`MemoryPartitionId::ExtendedVshell`] partition information.
    pub ext_vshell: MemoryPartitionBasicInfo,
}

/// Extra options to pass to [`sceKernelAllocMemoryBlock`].
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemoryBlockAllocOptions {
    pub size: SceSize,
}

/// Control block for UID
#[repr(C, packed)]
#[derive(Clone)]
pub struct UidControlBlock {
    pub parent0: *mut UidControlBlock,
    pub next_child: *mut UidControlBlock,
    /// The Uid kind
    pub kind: *mut UidControlBlock,
    pub uid: SceUid,
    pub name: *mut c_char,
    pub child_size: u8,
    pub size: u8,
    pub attribute: u16,
    pub next: UidControlBlockNext,
    pub parent1: *mut UidControlBlock,
    pub function_table: *mut UidLookupFunction,
}

/// Function pointer for [`UidLookupFunction`].
type UidFunction = unsafe extern "C" fn(
    uid: *mut UidControlBlock,
    uid_with_func: *mut UidControlBlock,
    func_id: u32,
    ap: VaList,
);

/// Lookup table of [`UidControlBlock`].
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UidLookupFunction {
    pub id: u32,
    pub func: UidFunction,
}

/// Inner union of [`UidControlBlock`].
#[repr(C)]
#[derive(Clone, Copy)]
pub union UidControlBlockNext {
    pub next: *mut UidControlBlock,
    pub num_child: i32,
}

/// The UID list representation.
#[repr(C)]
pub struct UidList {
    pub root: *mut UidControlBlock,
    pub root_kind: *mut UidControlBlock,
    pub basic: *mut UidControlBlock,
    pub count: SceSize,
}

/// The game information loaded to the system.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GameInfo {
    /// The size of the structure,
    pub size: SceSize,
    /// The bitflags that hold information what field was set.
    pub flags: GameInfoFlags,
    /// Raw UMD param SFO data.
    pub umd_param_sfo: [u8; 16],
    pub expect_umd_data: [u8; 16],
    pub qtgp2: [u8; 8],
    pub qtgp3: [u8; 16],
    pub allow_replace_umd: u32,
    pub title_id: [u8; 16],
    pub parental_level: u32,
    pub vsh_version: [u8; 8],
    pub umd_cache_on: u32,
    pub compiled_sdk_version: u32,
    pub compiler_version: u32,
    pub dnas: u32,
    pub utility_location: u32,
    pub vsh_bootfilename: [u8; 64],
    pub gamedata_id: [u8; 16],
    pub app_ver: [u8; 8],
    pub subscription_validity: [u8; 8],
    pub bootable: u32,
    pub opnssmp_ver: u32,
}

/// A bitmask value with the information to what fields are set in a [`GameInfo`].
#[bitflag(u32)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GameInfoFlags {
    /// [`GameInfo::umd_param_sfo`] was set.
    UmdParamSfo = 0x00001,
    /// [`GameInfo::qtgp2`] was set.
    Qtgp2 = 0x00004,
    /// [`GameInfo::qtgp3`] was set.
    Qtgp3 = 0x00008,
    /// [`GameInfo::allow_replace_umd`] was set.
    AllowReplaceUmd = 0x00010,
    /// [`GameInfo::umd_cache_on`] was set.
    UmdCacheOn = 0x00200,
    /// [`GameInfo::utility_location`] was set.
    UtilityLocation = 0x20000,
    /// [`GameInfo::vsh_bootfilename`] was set.
    VshBootfileName = 0x40000,
    /// [`GameInfo::subscription_validity`] was set.
    SubscriptionValidity = 0x80000,
    /// [`GameInfo::compiled_sdk_version`] was set.
    CompiledSdkVersion = 0x01000,
    /// [`GameInfo::compiler_version`] was set.
    CompilerVersion = 0x02000,
    /// [`GameInfo::dnas`] was set.
    Dnas  = 0x10000,
    /// [`GameInfo::title_id`], [`GameInfo::parental_level`], [`GameInfo::gamedata_id`],
    /// [`GameInfo::app_ver`], [`GameInfo::bootable`], [`GameInfo::opnssmp_ver`] and
    /// [`GameInfo::vsh_version`] were set.
    SfoData = 0x00100,
}

#[psp_stub(libname = "SysMemUserForUser", flags = 0x4000)]
extern "C" {
    /// Allocates a memory block from a memory partition.
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
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelAllocPartitionMemory(
        partition: MemoryPartitionId, name: *const u8, kind: MemoryBlockKind, size: SceSize,
        addr: SceSize,
    ) -> SceResult<MemoryBlockId>;

    /// Gets the address of a memory block.
    ///
    /// # Parameters
    ///
    /// `block_id`: The UID of the memory block.
    ///
    /// # Return Values
    ///
    /// Returns the lowest address belonging to the memory block.
    #[nid(0x9D9A5BA1)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceKernelGetBlockHeadAddr(block_id: MemoryBlockId) -> *mut c_void;

    /// Deallocates a memory block allocated with [`sceKernelAllocPartitionMemory`].
    ///
    /// # Parameters
    ///
    /// - `block_id`: The UID of the block to free.
    ///
    /// # Return value
    ///
    /// Returns an unknown value on success, an error value otherwise.
    #[nid(0xB6D61D02)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelFreePartitionMemory(block_id: MemoryBlockId) -> SceResult<u32>;

    /// Allocates a memory block from the main user partition.
    ///
    /// # Parameters
    ///
    /// - `name`: Name assigned to the new block. Only used for debug.
    /// - `kind`: Specifies how the block is allocated within the partition.
    /// - `size`: Size of the memory block, in bytes.
    /// - `options`: The options configuring the allocation behavior. If [`None`], the function will
    ///   assume default behavior.
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
        options: Option<&MemoryBlockAllocOptions>,
    ) -> SceResult<MemoryBlockId>;

    /// Gets the address of a memory block.
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

    /// Deallocates a memory block allocated with [`sceKernelAllocMemoryBlock`].
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
    #[cfg(not(feature = "kernel"))]
    pub fn sceKernelQueryMemoryInfo(
        addr: SceSize, partition_id: &mut SceIsize, block_id: &mut SceIsize,
    ) -> SceResult<()>;

    /// Gets the total size of the main user memory partition.
    ///
    /// # Return Value
    ///
    /// The total size of the main user memory partition in bytes.
    #[nid(0xACBD88CA)]
    pub fn sceKernelTotalMemSize() -> SceSize;

    /// Gets the total amount of free memory.
    ///
    /// # Return Value
    ///
    /// The total amount of free memory, in bytes.
    #[nid(0xF919F628)]
    pub fn sceKernelTotalFreeMemSize() -> SceSize;

    /// Gets the size of the largest free memory block.
    ///
    /// # Return Value
    ///
    /// The size of the largest free memory block, in bytes.
    #[nid(0xA291F107)]
    pub fn sceKernelMaxFreeMemSize() -> SceSize;

    /// Gets the firmware version.
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
    #[cfg(not(feature = "kernel"))]
    pub fn sceKernelDevkitVersion() -> u32;

    /// Sets the version of the SDK with which the caller was compiled.
    ///
    /// Version numbers are the same as for [`sceKernelDevkitVersion`].
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x7591C7DB)]
    pub fn sceKernelSetCompiledSdkVersion(version: u32) -> SceResult<()>;

    /// Gets the SDK version set with [`sceKernelSetCompiledSdkVersion`].
    ///
    /// # Return Value
    ///
    /// Returns the version number, or `0` if unset.
    #[nid(0xFC114573)]
    #[cfg(not(feature = "kernel"))]
    pub fn sceKernelGetCompiledSdkVersion() -> u32;

    /// Kernel printf function.
    ///
    /// # Parameters
    ///
    /// - `format`: The format string.
    /// - `...`: Variables to print.
    #[nid(0x13A5ABEF)]
    pub unsafe fn sceKernelPrintf(format: *const c_char, ...);

    #[nid(0xA6848DF8)]
    pub unsafe fn sceKernelSetUsersystemLibWork(
        cmd_list: *mut i32, func: GeListUpdateStallAddrLazy, lazy: *mut GeLazy,
    ) -> SceResult<()>;
}

// FIXME: Add missing functions (uncracked named function not included)
//
// These are more important:
//
// s32 sceKernelResizeMemoryBlock(SceUID id, s32 leftShift, s32 rightShift);
// s32 sceKernelJointMemoryBlock(SceUID id1, SceUID id2);
// s32 sceKernelSeparateMemoryBlock(SceUID id, u32 cutBefore, u32 size);
// s32 sceKernelQueryMemoryBlockInfo(SceUID id, SceSysmemMemoryBlockInfo *infoPtr);
// s32 sceKernelSizeLockMemoryBlock(SceUID id);
//
// void *sceKernelMemset(void *src, s8 c, u32 size);
// void *sceKernelMemset32(void *src, s32 c, u32 size);
// void *sceKernelMemmove(void *dst, void *src, u32 size);
// void *sceKernelMemmoveWithFill(void *dst, void *src, u32 size, s32 fill);
// void *sceKernelMemcpy(void *dst, const void *src, u32 n);
//
// Memory Operations:
//
// void sceKernelMemoryExtendSize(void);
// void sceKernelMemoryShrinkSize(void);
// u32 sceKernelMemoryOpenSize(void);
// void sceKernelMemoryCloseSize(u32 state);
//
// UID:
//
// s32 sceKernelCallUIDFunction(SceUID id, s32 funcId, ...);
// s32 sceKernelCallUIDObjFunction(SceSysmemUidCB *uid, s32 funcId, ...);
// int sceKernelLookupUIDFunction(SceSysmemUidCB *uid, int id, SceSysmemUidFunc *func,
//      SceSysmemUidCB **parentUidWithFunc);
// s32 sceKernelCallUIDObjCommonFunction(SceSysmemUidCB *uid, SceSysmemUidCB *uidWithFunc,
//      s32 funcId, va_list ap);
// int sceKernelCreateUIDtypeInherit(const char *parentName, const char *name, int size,
//      SceSysmemUidLookupFunc *funcTable, SceSysmemUidLookupFunc *metaFuncTable,
//      SceSysmemUidCB **uidTypeOut);
// int sceKernelCreateUID(SceSysmemUidCB *type, const char *name, char k1,
//      SceSysmemUidCB **outUid);
// SceUID sceKernelSearchUIDbyName(const char *name, SceUID typeId);
// int sceKernelCreateUIDtype(const char *name, int size, SceSysmemUidLookupFunc *funcTable,
//      SceSysmemUidLookupFunc *metaFuncTable, SceSysmemUidCB **uidTypeOut);
// s32 sceKernelDeleteUIDtype(SceSysmemUidCB *uid);
// s32 sceKernelGetUIDname(SceUID id, s32 len, char *out);
// s32 sceKernelRenameUID(SceUID id, const char *name);
// s32 sceKernelGetUIDtype(SceUID id);
// s32 sceKernelIsKindOf(SceSysmemUidCB *uid, SceSysmemUidCB *type);
// s32 sceKernelPrintUidListAll(void);
//
// s32 sceKernelIsHold(SceSysmemUidCB *uid0, SceSysmemUidCB *uid1);
// s32 sceKernelHoldUID(SceUID id0, SceUID id1);
// s32 sceKernelReleaseUID(SceUID id0, SceUID id1);
//
// Debugging (disabled in release) so not very important
//
// s32 sceKernelApiEvaluationInit();
// s32 sceKernelRegisterApiEvaluation();
// s32 sceKernelApiEvaliationAddData();
// s32 sceKernelApiEvaluationReport();
// s32 sceKernelSetGcovFunction();
// s32 sceKernelCallGcovFunction();
// s32 sceKernelSetGprofFunction();
// s32 sceKernelCallGprofFunction();
// int sceKernelCheckDebugHandler();
#[cfg(feature = "kernel")]
#[psp_stub(libname = "SysMemForKernel", flags = 0x0001)]
extern "C" {
    /// Allocates a memory block from a memory partition.
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
    #[nid(if cfg!(feature = "psp_660") { 0x7158CE7E }
        else if cfg!(feature = "psp_630") { 0x4621A9CC }
        else if cfg!(feature = "psp_600") { 0xE5E18A99 }
        else if cfg!(feature = "psp_570") { 0xEA349DC6 }
        else if cfg!(feature = "psp_500") { 0x5630F321 }
        else if cfg!(feature = "psp_420") { 0x2349884C }
        else if cfg!(feature = "psp_395") { 0xE9E24E73 }
        else if cfg!(feature = "psp_380") { 0x135CB831 }
        else { 0x237DBD4F }
    )]
    pub unsafe fn sceKernelAllocPartitionMemory(
        partition: MemoryPartitionId, name: *const u8, kind: MemoryBlockKind, size: SceSize,
        addr: SceSize,
    ) -> SceResult<MemoryBlockId>;

    /// Gets the address of a memory block.
    ///
    /// # Parameters
    ///
    /// `block_id`: The UID of the memory block.
    ///
    /// # Return Values
    ///
    /// Returns the lowest address belonging to the memory block.
    #[nid(if cfg!(feature = "psp_660") { 0xF12A62F7 }
        else if cfg!(feature = "psp_630") { 0x52B54B93 }
        else if cfg!(feature = "psp_600") { 0xFEB5C72B }
        else if cfg!(feature = "psp_570") { 0x2BDA1AC9 }
        else if cfg!(feature = "psp_500") { 0x950BCB31 }
        else if cfg!(feature = "psp_420") { 0x9E55F4DA }
        else if cfg!(feature = "psp_395") { 0x5136926D }
        else if cfg!(feature = "psp_380") { 0xBC3AFFF2 }
        else { 0x9D9A5BA1 }
    )]
    pub fn sceKernelGetBlockHeadAddr(block_id: MemoryBlockId) -> *mut c_void;

    /// Deallocates a memory block allocated with [`sceKernelAllocPartitionMemory`].
    ///
    /// # Parameters
    ///
    /// - `block_id`: The UID of the block to free.
    ///
    /// # Return value
    ///
    /// Returns an unknown value on success, an error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xC1A26C6F }
        else if cfg!(feature = "psp_630") { 0x8FDAFC4C }
        else if cfg!(feature = "psp_600") { 0x093DE56A }
        else if cfg!(feature = "psp_570") { 0xE8120E5C }
        else if cfg!(feature = "psp_500") { 0xAFBE8876 }
        else if cfg!(feature = "psp_420") { 0x3B6F05DC }
        else if cfg!(feature = "psp_395") { 0x8B356F2A }
        else if cfg!(feature = "psp_380") { 0x40C91389 }
        else { 0xB6D61D02 }
    )]
    pub unsafe fn sceKernelFreePartitionMemory(block_id: MemoryBlockId) -> SceResult<u32>;

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
    #[nid(if cfg!(feature = "psp_660") { 0xFAF29F34 }
        else if cfg!(feature = "psp_630") { 0xE5799DEE }
        else if cfg!(feature = "psp_600") { 0x442F5709 }
        else if cfg!(feature = "psp_570") { 0xD8C6965E }
        else if cfg!(feature = "psp_500") { 0x22AD1BD6 }
        else if cfg!(feature = "psp_420") { 0xB7FB5272 }
        else if cfg!(feature = "psp_395") { 0xDB6C07E8 }
        else if cfg!(feature = "psp_380") { 0x226C2C30 }
        else { 0x2A3E5280 }
    )]
    pub fn sceKernelQueryMemoryInfo(
        addr: SceSize, partition_id: &mut SceIsize, block_id: &mut SceIsize,
    ) -> SceResult<()>;

    /// Gets the firmware version.
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
    #[nid(if cfg!(feature = "psp_660") { 0xC886B169 }
        else if cfg!(feature = "psp_630") { 0x5E8DCA05 }
        else if cfg!(feature = "psp_600") { 0xFE9BC18B }
        else if cfg!(feature = "psp_570") { 0xFAD96D1C }
        else if cfg!(feature = "psp_500") { 0x7165B995 }
        else if cfg!(feature = "psp_420") { 0x8ACC368A }
        else if cfg!(feature = "psp_395") { 0x764C9811 }
        else if cfg!(feature = "psp_380") { 0xEE1718BC }
        else { 0x3FC9AE6A }
    )]
    pub fn sceKernelDevkitVersion() -> u32;

    /// Gets the SDK version set with [`sceKernelSetCompiledSdkVersion`].
    ///
    /// # Return Value
    ///
    /// Returns the version number, or `0` if unset.
    #[nid(if cfg!(feature = "psp_660") { 0xB4F00CB5 }
        else if cfg!(feature = "psp_630") { 0xF0E0AB7A }
        else if cfg!(feature = "psp_600") { 0xB9796F69 }
        else if cfg!(feature = "psp_570") { 0xF3A2E92B }
        else if cfg!(feature = "psp_500") { 0x2CFF6F90 }
        else if cfg!(feature = "psp_420") { 0x1DFCB40D }
        else if cfg!(feature = "psp_395") { 0x12720EDA }
        else if cfg!(feature = "psp_380") { 0x9110439F }
        else { 0xFC114573 }
    )]
    pub fn sceKernelGetCompiledSdkVersion() -> u32;

    /// Requests the information of a partition.
    ///
    /// # Parameters
    ///
    /// - `partition_id`: The ID of the requested memory partition information.
    /// - `info` **[[Out parameter]]**: A reference to the struct to receive the partition
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xC4EEAF20 }
        else if cfg!(feature = "psp_630") { 0xF5E82409 }
        else if cfg!(feature = "psp_600") { 0xE057A674 }
        else if cfg!(feature = "psp_570") { 0x81E4AB5E }
        else if cfg!(feature = "psp_500") { 0x940FCB99 }
        else if cfg!(feature = "psp_420") { 0x305864CE }
        else if cfg!(feature = "psp_395") { 0x78DAF618 }
        else if cfg!(feature = "psp_380") { 0xECFE305B }
        else { 0x55A40B2C }
    )]
    pub fn sceKernelQueryMemoryPartitionInfo(
        partition_id: MemoryPartitionId, info: &mut MemoryPartitionInfo,
    ) -> SceResult<()>;

    /// Gets the total amount of free memory of a memory partition.
    ///
    /// # Parameters
    ///
    /// - `partition_id`: The ID of the requested memory partition.
    ///
    /// # Return Value
    ///
    /// The total amount of free memory in the requested memory partition, in bytes.
    #[nid(if cfg!(feature = "psp_660") { 0x0115B0F8 }
        else if cfg!(feature = "psp_630") { 0x7BE9653E }
        else if cfg!(feature = "psp_600") { 0x35588461 }
        else if cfg!(feature = "psp_570") { 0xB2D14F3B }
        else if cfg!(feature = "psp_500") { 0x182E3565 }
        else if cfg!(feature = "psp_420") { 0x869EBE58 }
        else if cfg!(feature = "psp_395") { 0x205AD4BC }
        else if cfg!(feature = "psp_380") { 0x40F41273 }
        else { 0x9697CD32 }
    )]
    pub fn sceKernelPartitionTotalFreeMemSize(partition_id: MemoryPartitionId) -> SceSize;

    /// Gets the size of the largest free memory block of a memory partition.
    ///
    /// # Parameters
    ///
    /// - `partition_id`: The ID of the requested memory partition.
    ///
    /// # Return Value
    ///
    /// The size of the largest free memory block in the requested memory partition, in bytes.
    #[nid(if cfg!(feature = "psp_660") { 0x13F4A0DE }
        else if cfg!(feature = "psp_630") { 0xE10F21CF }
        else if cfg!(feature = "psp_600") { 0xA25BF7A1 }
        else if cfg!(feature = "psp_570") { 0xBCDA7C0A }
        else if cfg!(feature = "psp_500") { 0x7A225A27 }
        else if cfg!(feature = "psp_420") { 0xE4E881B1 }
        else if cfg!(feature = "psp_395") { 0x651733CE }
        else if cfg!(feature = "psp_380") { 0x8C2C0E71 }
        else { 0xE6581468 }
    )]
    pub fn sceKernelPartitionMaxFreeMemSize(partition_id: MemoryPartitionId) -> SceSize;

    /// Fills the free blocks with a value.
    ///
    /// # Parameters
    ///
    /// - `partition_id`: The ID of the requested memory partition.
    /// - `val`: The value to fill the free blocks.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xEA1CABF1 }
        else if cfg!(feature = "psp_630") { 0x0FCC1DB5 }
        else if cfg!(feature = "psp_600") { 0x22CD6BE7 }
        else if cfg!(feature = "psp_570") { 0x794ADE01 }
        else if cfg!(feature = "psp_500") { 0xC71F5C50 }
        else if cfg!(feature = "psp_420") { 0x82130FB4 }
        else if cfg!(feature = "psp_395") { 0xEE6084E0 }
        else if cfg!(feature = "psp_380") { 0x8C0EEB1A }
        else { 0xA2A65F0E }
    )]
    pub fn sceKernelFillFreeBlock(partition_id: MemoryPartitionId, val: u32) -> SceResult<()>;

    /// Sets the protection of a block of DDR memory.
    ///
    /// # Parameters
    ///
    /// - `addr`: The address to the start to the block to set protection.
    /// - `size`: The size of the block to set protection.
    /// - `prot`: The protection bitmask
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x83B5226D }
        else if cfg!(feature = "psp_630") { 0x00E9A04A }
        else if cfg!(feature = "psp_600") { 0x31DFE03F }
        else if cfg!(feature = "psp_570") { 0xA7743EA4 }
        else if cfg!(feature = "psp_500") { 0x73546131 }
        else if cfg!(feature = "psp_420") { 0x8B66B075 }
        else if cfg!(feature = "psp_395") { 0x6E085C78 }
        else if cfg!(feature = "psp_380") { 0xBBD786DD }
        else { 0xB2C7AA36 }
    )]
    pub unsafe fn sceKernelSetDdrMemoryProtection(
        addr: SceSize, size: SceSize, prot: u32,
    ) -> SceResult<()>;

    /// Creates a heap.
    ///
    /// # Parameters
    ///
    /// - `partition_id`: The ID of the memory partition where to create the heap.
    /// - `size`: The size of the heap to create.
    /// - `flags`: Flags that change heap creation behavior. Check [`HeapCreateFlag`] to know more.
    /// - `name`: Name assigned to the new heap. Only used for debug.
    ///
    /// # Return Value
    ///
    /// The new heap UID on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x58148F07 }
        else if cfg!(feature = "psp_630") { 0xAF85EB1B }
        else if cfg!(feature = "psp_600") { 0xC6A782DA }
        else if cfg!(feature = "psp_570") { 0x37878371 }
        else if cfg!(feature = "psp_500") { 0x4F47A4AD }
        else if cfg!(feature = "psp_420") { 0x4F92E72C }
        else if cfg!(feature = "psp_395") { 0x226457D6 }
        else if cfg!(feature = "psp_380") { 0xFD2A7114 }
        else { 0x1C1FBFE7 }
    )]
    pub unsafe fn sceKernelCreateHeap(
        partition_id: MemoryPartitionId, size: SceSize, flags: HeapCreateFlag, name: *const c_char,
    ) -> SceResult<HeapId>;


    /// Allocates a memory block from a heap.
    ///
    /// # Parameters
    ///
    /// - `heap_id`: The UID for a heap.
    /// - `size`: The size of the memory block to allocate.
    ///
    /// # Return Value
    ///
    /// Returns a pointer to the start of the allocated memory block or `null` value if the
    /// allocation failed.
    #[nid(if cfg!(feature = "psp_660") { 0x23D81675 }
        else if cfg!(feature = "psp_630") { 0x6D161EE2 }
        else if cfg!(feature = "psp_600") { 0x96BFE779 }
        else if cfg!(feature = "psp_570") { 0x6CC6A838 }
        else if cfg!(feature = "psp_500") { 0xFDF96599 }
        else if cfg!(feature = "psp_420") { 0x17AA94CD }
        else if cfg!(feature = "psp_395") { 0xED52C3A2 }
        else if cfg!(feature = "psp_380") { 0x6986B4E3 }
        else { 0x636C953B }
    )]
    pub unsafe fn sceKernelAllocHeapMemory(heap_id: HeapId, size: SceSize) -> *mut c_void;

    /// Allocates a memory block from a heap configured with options.
    ///
    /// # Parameters
    ///
    /// - `heap_id`: The UID for a heap.
    /// - `size`: The size of the memory block to allocate.
    /// - `options`: The options configuring the allocation behavior. If [`None`], the function will
    ///   assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns a pointer to the start of the allocated memory block or `null` value if the
    /// allocation failed.
    #[nid(if cfg!(feature = "psp_660") { 0xF2284ECC }
        else if cfg!(feature = "psp_630") { 0x25C56981 }
        else if cfg!(feature = "psp_600") { 0x191C6F98 }
        else if cfg!(feature = "psp_570") { 0x0B3AC38F }
        else if cfg!(feature = "psp_500") { 0x36960E64 }
        else if cfg!(feature = "psp_420") { 0xCFE8F6A1 }
        else if cfg!(feature = "psp_395") { 0x8E2EA487 }
        else if cfg!(feature = "psp_380") { 0x3B2A4D7F }
        else { 0xEB7A74DB }
    )]
    pub unsafe fn sceKernelAllocHeapMemoryWithOption(
        heap_id: HeapId,
        size: SceSize,
        // Original is `*const HeapAllocOptions`
        // We know the implementation, that Can take NULL or valid ptr (reference) to
        // `HeapAllocOptions` and only read the options. Knowing that, we can safely
        // use `Option<&HeapAllocOptions>`
        options: Option<&HeapAllocOptions>,
    ) -> *mut c_void;

    /// Deallocates a memory block allocated from a heap.
    ///
    /// # Parameters
    ///
    /// - `heap_id`: The UID for a heap.
    /// - `mem_block` **[[Out parameter]]**: The memory block to deallocate from a heap.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x87C2AB85 }
        else if cfg!(feature = "psp_630") { 0xDB836ADB }
        else if cfg!(feature = "psp_600") { 0xC2A8C491 }
        else if cfg!(feature = "psp_570") { 0x253FA9A4 }
        else if cfg!(feature = "psp_500") { 0xAB439400 }
        else if cfg!(feature = "psp_420") { 0xE3EDC89C }
        else if cfg!(feature = "psp_395") { 0x27CDFCC9 }
        else if cfg!(feature = "psp_380") { 0xB6082F0D }
        else { 0x7B749390 }
    )]
    pub unsafe fn sceKernelFreeHeapMemory(heap_id: HeapId, mem_block: *mut c_void)
        -> SceResult<()>;

    /// Deletes a heap associated to a heap UID.
    ///
    /// # Parameters
    ///
    /// - `heap_id`: The UID for a heap.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xDD6512D0 }
        else if cfg!(feature = "psp_630") { 0xF9475C1A }
        else if cfg!(feature = "psp_600") { 0x9BAA857E }
        else if cfg!(feature = "psp_570") { 0xCEB7BA62 }
        else if cfg!(feature = "psp_500") { 0x386D0EB3 }
        else if cfg!(feature = "psp_420") { 0x5450808B }
        else if cfg!(feature = "psp_395") { 0x7A5B790E }
        else if cfg!(feature = "psp_380") { 0x079C9E14 }
        else { 0xC9805775 }
    )]
    pub unsafe fn sceKernelDeleteHeap(heap_id: HeapId) -> SceResult<()>;

    /// Gets the total amount of free memory of a heap.
    ///
    /// # Parameters
    ///
    /// - `heap_id`: The UID for a heap.
    ///
    /// # Return Value
    ///
    /// Returns the total amount of free memory of a heap, in bytes, on success, error value
    /// otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x01810023 }
        else if cfg!(feature = "psp_630") { 0x61F1BD6F }
        else if cfg!(feature = "psp_600") { 0xA60B6316 }
        else if cfg!(feature = "psp_570") { 0xFED328EF }
        else if cfg!(feature = "psp_500") { 0xC53F4D35 }
        else if cfg!(feature = "psp_420") { 0xCE83B972 }
        else if cfg!(feature = "psp_395") { 0x2F0022EA }
        else if cfg!(feature = "psp_380") { 0x3DBBB447 }
        else { 0xA823047E }
    )]
    pub fn sceKernelHeapTotalFreeSize(heap_id: HeapId) -> SceResult<SceSize>;

    /// Gets the information of a heap object.
    ///
    /// # Parameters
    ///
    /// - `heap_id`: The UID for a heap.
    /// - `info` **[[Out parameter]]**: A reference to a [`HeapInfo`] structure to receive the
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x79BD1975 }
        else if cfg!(feature = "psp_630") { 0x54DEFDA7 }
        else if cfg!(feature = "psp_600") { 0x6FDEF86F }
        else if cfg!(feature = "psp_570") { 0xE9AA9A96 }
        else if cfg!(feature = "psp_500") { 0x28CDFAEC }
        else if cfg!(feature = "psp_420") { 0x74411250 }
        else if cfg!(feature = "psp_395") { 0x655EA559 }
        else if cfg!(feature = "psp_380") { 0x7EF4BDCB }
        else { 0x002BA296 }
    )]
    pub fn sceKernelQueryHeapInfo(heap_id: HeapId, info: &mut HeapInfo) -> SceResult<()>;

    /// Get the low heap information
    ///
    /// # Parameters
    ///
    /// - `heap_block` **[[In parameter]]**: A pointer to the heap block to get the low heap
    ///   information,
    /// - `info` **[[Out parameter]]**: A reference to receive the information.
    ///
    /// # Return Value
    ///
    /// Returns the block count on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x476B244F }
        else if cfg!(feature = "psp_630") { 0xD94A5183 }
        else if cfg!(feature = "psp_600") { 0x3988900F }
        else if cfg!(feature = "psp_570") { 0x9ED920A5 }
        else if cfg!(feature = "psp_500") { 0x3221FDE9 }
        else if cfg!(feature = "psp_420") { 0xBC733E1C }
        else if cfg!(feature = "psp_395") { 0x3443547F }
        else if cfg!(feature = "psp_380") { 0x88F6A684 }
        else { 0x03808C51 }
    )]
    pub unsafe fn sceKernelQueryLowheapInfo(
        heap_block: *mut HeapBlock, info: &mut LowHeapInfo,
    ) -> SceResult<SceSize>;

    /// Gets the hardware model of PSP.
    ///
    /// # Return Value
    ///
    /// The hardware model of PSP.
    #[nid(if cfg!(feature = "psp_660") { 0x07C586A1 }
        else if cfg!(feature = "psp_630") { 0x458A70B5 }
        else if cfg!(feature = "psp_600") { 0x864EBFD7 }
        else if cfg!(feature = "psp_570") { 0xBD444810 }
        else if cfg!(feature = "psp_500") { 0xDA07DC6E }
        else if cfg!(feature = "psp_420") { 0x4B666E99 }
        else if cfg!(feature = "psp_395") { 0x538068EF }
        else if cfg!(feature = "psp_380") { 0xA3B0B6BC }
        else { 0x6373995D }
    )]
    pub fn sceKernelGetModel() -> PspHardwareModel;

    /// Gets the current status of the system.
    ///
    /// ## Known Values
    /// - `0x20000`: Power unlock done.
    /// - `0x40000`: RunExecForThread prepared.
    /// - `0x40020`: runReboot is read and about to to start.
    #[nid(if cfg!(feature = "psp_660") { 0x36C503A9 }
        else if cfg!(feature = "psp_630") { 0x6D336C84 }
        else if cfg!(feature = "psp_600") { 0x2E0F38E5 }
        else if cfg!(feature = "psp_570") { 0xD9B3E1CE }
        else if cfg!(feature = "psp_500") { 0xC3EB9D57 }
        else if cfg!(feature = "psp_420") { 0x74D7B1BF }
        else if cfg!(feature = "psp_395") { 0x4823B9D9 }
        else if cfg!(feature = "psp_380") { 0xA48F9782 }
        else { 0x452E3696 }
    )]
    pub fn sceKernelGetSystemStatus() -> u32;

    /// Sets the status of the system.
    ///
    /// # Parameters
    ///
    /// - `new_status`: The new status value to set.
    ///
    /// # Returns Value
    ///
    /// Returns the last status value of the system.
    ///
    /// # Safety
    ///
    /// Changing the system status can cause issues in system behavior.
    #[nid(if cfg!(feature = "psp_660") { 0x521AC5A4 }
        else if cfg!(feature = "psp_630") { 0x604C20C4 }
        else if cfg!(feature = "psp_600") { 0x98ACDE01 }
        else if cfg!(feature = "psp_570") { 0x300BF4A3 }
        else if cfg!(feature = "psp_500") { 0x578ED40F }
        else if cfg!(feature = "psp_420") { 0x795B57DA }
        else if cfg!(feature = "psp_395") { 0x15645B7F }
        else if cfg!(feature = "psp_380") { 0xD26FFD63 }
        else { 0x95F5E8DA }
    )]
    pub unsafe fn sceKernelSetSystemStatus(new_status: u32) -> u32;

    /// Gets the information of the game.
    ///
    /// # Return Value
    ///
    /// Returns a pointer to the global game information.
    #[nid(if cfg!(feature = "psp_660") { 0xEF29061C }
        else if cfg!(feature = "psp_630") { 0x38F4EBDC }
        else if cfg!(feature = "psp_600") { 0x2F15F149 }
        else if cfg!(feature = "psp_570") { 0x2694D47F }
        else if cfg!(feature = "psp_500") { 0x0D547E7F }
        else if cfg!(feature = "psp_420") { 0xB7BB3534 }
        else if cfg!(feature = "psp_395") { 0xB3C487AF }
        else if cfg!(feature = "psp_380") { 0xB1634112 }
        else { 0xCD617A94 }
    )]
    pub fn sceKernelGetGameInfo() -> NonNull<GameInfo>;

    /// Copy the global game information to a given local structure.
    ///
    /// # Parameters
    ///
    /// - `info` **[[Out parameter]]**: A pointer to a local [`GameInfo`] structure to receive the
    ///   information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xB9B7281A }
        else if cfg!(feature = "psp_630") { 0xD9B4E550 }
        else if cfg!(feature = "psp_600") { 0xCA852144 }
        else if cfg!(feature = "psp_570") { 0x4B5A92C7 }
        else if cfg!(feature = "psp_500") { 0x9D87D5B3 }
        else if cfg!(feature = "psp_420") { 0x0A1A5C1C }
        else if cfg!(feature = "psp_395") { 0x8B26A91E }
        else if cfg!(feature = "psp_380") { 0xAFDB003A }
        else { 0x97B18FA8 }
    )]
    pub unsafe fn sceKernelCopyGameInfo(info: *mut GameInfo) -> SceResult<()>;

    /// Sets the UMD Param SFO on the global [`GameInfo`].
    ///
    /// # Parameters
    ///
    /// - `umd_param_sfo` **[[In parameter]]**: The UMD Param SFO to set. If the pointer is `null`,
    ///   it resets the data in the global game information.
    #[nid(if cfg!(feature = "psp_660") { 0xF3BDB718 }
        else if cfg!(feature = "psp_630") { 0x9C6BBA4B }
        else if cfg!(feature = "psp_600") { 0x1DFFBF56 }
        else if cfg!(feature = "psp_570") { 0xD7E47ED6 }
        else if cfg!(feature = "psp_500") { 0xC2460DD5 }
        else if cfg!(feature = "psp_420") { 0xFD35BB74 }
        else if cfg!(feature = "psp_395") { 0xC3A6C02A }
        else if cfg!(feature = "psp_380") { 0xB8637A4B }
        else { 0xF91FE6AA }
    )]
    pub unsafe fn sceKernelSetParamSfo(umd_param_sfo: *const [u8; 16]);

    /// Gets the QTGP2 value from the global [`GameInfo`].
    ///
    /// # Parameters
    ///
    /// - `qtgp2` **[[Out parameter]]**: A reference to an array to receive the data.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x474CA24F }
        else if cfg!(feature = "psp_630") { 0x5B7174DF }
        else if cfg!(feature = "psp_600") { 0x040CDA4C }
        else if cfg!(feature = "psp_570") { 0x4FB80176 }
        else if cfg!(feature = "psp_500") { 0x0171E32C }
        else if cfg!(feature = "psp_420") { 0xF4C378B7 }
        else if cfg!(feature = "psp_395") { 0x4C55435B }
        else if cfg!(feature = "psp_380") { 0x9AB85DA9 }
        else { 0xCE8D3DB3 }
    )]
    pub fn sceKernelGetQTGP2(qtgp2: &mut [u8; 8]) -> SceResult<()>;

    /// Sets the QTGP2 value on the global [`GameInfo`].
    ///
    /// # Parameters
    ///
    /// - `qtgp2` **[[Out parameter]]**: A pointer to the QTGP2 value to set. If the pointer is
    ///   `null`, it resets the data in the global game information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xA03CB480 }
        else if cfg!(feature = "psp_630") { 0x9E5B1ACB }
        else if cfg!(feature = "psp_600") { 0xD391637C }
        else if cfg!(feature = "psp_570") { 0x1FE5F49C }
        else if cfg!(feature = "psp_500") { 0x58911B8B }
        else if cfg!(feature = "psp_420") { 0x4DBD3833 }
        else if cfg!(feature = "psp_395") { 0x797E7D55 }
        else if cfg!(feature = "psp_380") { 0xCDBFF97E }
        else { 0x55E4719C }
    )]
    pub unsafe fn sceKernelSetQTGP2(qtgp2: *mut [u8; 8]) -> SceResult<()>;

    /// Gets the QTGP3 value from the global [`GameInfo`].
    ///
    /// # Parameters
    ///
    /// - `qtgp3` **[[Out parameter]]**: A reference to an array to receive the data.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x43E0A861 }
        else if cfg!(feature = "psp_630") { 0x80C7689B }
        else if cfg!(feature = "psp_600") { 0x2F756CC8 }
        else if cfg!(feature = "psp_570") { 0x193E81C1 }
        else if cfg!(feature = "psp_500") { 0x80BD86BE }
        else if cfg!(feature = "psp_420") { 0x1EA7EB34 }
        else if cfg!(feature = "psp_395") { 0x2C086CA7 }
        else if cfg!(feature = "psp_380") { 0xB015F84F }
        else { 0x6D8E0CDF }
    )]
    pub fn sceKernelGetQTGP3(qtgp3: &mut [u8; 16]) -> SceResult<()>;

    /// Sets the QTGP3 value on the global [`GameInfo`].
    ///
    /// # Parameters
    ///
    /// - `qtgp3` **[[Out parameter]]**: A pointer to the QTGP3 value to set. If the pointer is
    ///   `null`, it resets the data in the global game information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xFC639A2B }
        else if cfg!(feature = "psp_630") { 0x9F154FA1 }
        else if cfg!(feature = "psp_600") { 0xB00D1EC2 }
        else if cfg!(feature = "psp_570") { 0x699F8B1A }
        else if cfg!(feature = "psp_500") { 0xA7188E84 }
        else if cfg!(feature = "psp_420") { 0xCF66F072 }
        else if cfg!(feature = "psp_395") { 0x887ED503 }
        else if cfg!(feature = "psp_380") { 0x1385A8F2 }
        else { 0xC7E57B9C }
    )]
    pub unsafe fn sceKernelSetQTGP3(qtgp3: *mut [u8; 16]) -> SceResult<()>;

    /// Gets if replacing UMD is allowed from the global [`GameInfo`].
    ///
    /// # Parameters
    ///
    /// - `allow_replace_umd` **[[Out parameter]]**: A reference to receive the data.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x4972F9D1 }
        else if cfg!(feature = "psp_630") { 0x966EB2BE }
        else if cfg!(feature = "psp_600") { 0x1205AE0C }
        else if cfg!(feature = "psp_570") { 0xDB5D4EFB }
        else if cfg!(feature = "psp_500") { 0x76DD5020 }
        else if cfg!(feature = "psp_420") { 0x78A5301C }
        else if cfg!(feature = "psp_395") { 0x39240A0C }
        else if cfg!(feature = "psp_380") { 0xEE010E3A }
        else { 0xA262FEF0 }
    )]
    pub fn sceKernelGetAllowReplaceUmd(allow_replace_umd: &mut u32) -> SceResult<()>;

    /// Sets if replacing UMD is allowed on the global [`GameInfo`].
    ///
    /// # Parameters
    ///
    /// - `allow_replace_umd`: The value to set.
    #[nid(if cfg!(feature = "psp_660") { 0xF19BA38D }
        else if cfg!(feature = "psp_630") { 0x72DB42EC }
        else if cfg!(feature = "psp_600") { 0x7EF1DC3C }
        else if cfg!(feature = "psp_570") { 0xC8510803 }
        else if cfg!(feature = "psp_500") { 0xFE94D4E3 }
        else if cfg!(feature = "psp_420") { 0x0112B75F }
        else if cfg!(feature = "psp_395") { 0x60B26714 }
        else if cfg!(feature = "psp_380") { 0x8B9178EF }
        else { 0xCBB05241 }
    )]
    pub fn sceKernelSetAllowReplaceUmd(allow_replace_umd: u32);

    /// Gets the DNAS information from the globel [`GameInfo`].
    ///
    /// # Return Value
    ///
    /// Returns the DNAS information, `0` if not set.
    #[nid(if cfg!(feature = "psp_660") { 0xBFD53FB7 }
        else if cfg!(feature = "psp_630") { 0xF7125FA4 }
        else if cfg!(feature = "psp_600") { 0xBD71F23A }
        else if cfg!(feature = "psp_570") { 0x88D5CE7E }
        else if cfg!(feature = "psp_500") { 0x3E0ED0E3 }
        else if cfg!(feature = "psp_420") { 0xE88364A3 }
        else if cfg!(feature = "psp_395") { 0xC6FABB66 }
        else if cfg!(feature = "psp_380") { 0xDB9BE041 }
        else { 0x7ECBDBD9 }
    )]
    pub fn sceKernelGetDNAS() -> u32;

    /// Sets the DNAS information on the global [`GameInfo`].
    ///
    /// # Parameters
    ///
    /// - `dnas`: The DNAS information to set.
    #[nid(if cfg!(feature = "psp_660") { 0x982A4779 }
        else if cfg!(feature = "psp_630") { 0xED6B1D87 }
        else if cfg!(feature = "psp_600") { 0xDCFF8AE5 }
        else if cfg!(feature = "psp_570") { 0x16470B81 }
        else if cfg!(feature = "psp_500") { 0x674A325B }
        else if cfg!(feature = "psp_420") { 0xF46AD7A3 }
        else if cfg!(feature = "psp_395") { 0xC5894250 }
        else if cfg!(feature = "psp_380") { 0x06BB6385 }
        else { 0x9C304ED7 }
    )]
    pub fn sceKernelSetDNAS(dnas: u32);

    /// Sets if UMD cache is on is allowed from the global [`GameInfo`].
    ///
    /// # Parameters
    ///
    /// - `umd_cache_on`: The value to set.
    #[nid(0x1404C1AA)]
    #[cfg(feature = "psp_660")]
    pub fn sceKernelSetUmdCacheOn(umd_cache_on: u32);

    /// Get a UID control block.
    ///
    /// # Parameters
    ///
    /// - `id`: The UID to get the information
    /// - `uid_control_block` **[[Out parameter]]**: A pointer to receive the information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xC90B0992 }
        else if cfg!(feature = "psp_630") { 0x3DE3C6DF }
        else if cfg!(feature = "psp_600") { 0xF5780DAA }
        else if cfg!(feature = "psp_570") { 0x9FA23D35 }
        else if cfg!(feature = "psp_500") { 0xD7E24299 }
        else if cfg!(feature = "psp_420") { 0x97337E44 }
        else if cfg!(feature = "psp_395") { 0x523E300A }
        else if cfg!(feature = "psp_380") { 0xE151223E }
        else { 0xCF4DE78C }
    )]
    pub unsafe fn sceKernelGetUIDcontrolBlock(
        id: SceUid, uid_control_block: NonNull<*mut UidControlBlock>,
    ) -> SceResult<()>;


    /// Gets a UID control block on a particular kind.
    ///
    /// # Parameters
    ///
    /// - `id`: The UID to get the information
    /// - `kind`: A pointer to the kind UID block.
    /// - `uid_control_block` **[[Out parameter]]**: A pointer to receive the information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x44BDF332 }
        else if cfg!(feature = "psp_630") { 0xCB9582C4 }
        else if cfg!(feature = "psp_600") { 0xD17C6005 }
        else if cfg!(feature = "psp_570") { 0x7CE7BB12 }
        else if cfg!(feature = "psp_500") { 0x18DE11D5 }
        else if cfg!(feature = "psp_420") { 0x22F01719 }
        else if cfg!(feature = "psp_395") { 0x8BA095E5 }
        else if cfg!(feature = "psp_380") { 0x120433CF }
        else { 0x41FFC7F9 }
    )]
    pub unsafe fn sceKernelGetUIDcontrolBlockWithType(
        id: SceUid, kind: NonNull<UidControlBlock>,
        uid_control_block: NonNull<*mut UidControlBlock>,
    ) -> SceResult<()>;

    /// Gets the root of the UID list.
    ///
    /// # Return Value
    ///
    /// Returns a pointer to the global UID root.
    #[nid(if cfg!(feature = "psp_660") { 0xAC12F678 }
        else if cfg!(feature = "psp_630") { 0x41E50FA6 }
        else if cfg!(feature = "psp_600") { 0xCCD58B8B }
        else if cfg!(feature = "psp_570") { 0x28FCE6A4 }
        else if cfg!(feature = "psp_500") { 0x213C3E27 }
        else if cfg!(feature = "psp_420") { 0x0899FBF8 }
        else if cfg!(feature = "psp_395") { 0x2A2E7057 }
        else if cfg!(feature = "psp_380") { 0x54247DD8 }
        else { 0x536AD5E1 }
    )]
    pub fn sceKernelGetUidmanCB() -> NonNull<UidList>;

    /// Deletes a UID.
    ///
    /// # Parameters
    ///
    /// - `id`: The UID to delete.
    ///
    /// # Return Values
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x361F0F88 }
        else if cfg!(feature = "psp_630") { 0x5E58B140 }
        else if cfg!(feature = "psp_600") { 0x6C00F0CC }
        else if cfg!(feature = "psp_570") { 0x283B0E0A }
        else if cfg!(feature = "psp_500") { 0xF8F2B3A6 }
        else if cfg!(feature = "psp_420") { 0xA6471B40 }
        else if cfg!(feature = "psp_395") { 0xCDE08426 }
        else if cfg!(feature = "psp_380") { 0xC80E5BE1 }
        else { 0x8F20C4C0 }
    )]
    pub unsafe fn sceKernelDeleteUID(id: SceUid) -> SceResult<()>;

    /// Gets the initial system random value.
    #[nid(if cfg!(feature = "psp_660") { 0x4A325AA0 }
        else if cfg!(feature = "psp_630") { 0x48F77207 }
        else if cfg!(feature = "psp_600") { 0x3B560EE6 }
        else if cfg!(feature = "psp_570") { 0x7528C8B1 }
        else if cfg!(feature = "psp_500") { 0xC40E2333 }
        else if cfg!(feature = "psp_420") { 0x643160C0 }
        else if cfg!(feature = "psp_395") { 0x23D6BCB6 }
        else if cfg!(feature = "psp_380") { 0x9E7265D9 }
        else { 0x38495D84 }
    )]
    pub fn sceKernelGetInitialRandomValue() -> u32;

    /// Sets the reboot kernel function.
    ///
    /// # Parameters
    ///
    /// - `reboot_kernel_fn`: The function pointer for the reboot kernel function
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x96A3CE2C }
        else if cfg!(feature = "psp_630") { 0xE0A74F2D }
        else if cfg!(feature = "psp_600") { 0xC5485286 }
        else if cfg!(feature = "psp_570") { 0x5BD1775C }
        else if cfg!(feature = "psp_500") { 0x1D849D0E }
        else if cfg!(feature = "psp_420") { 0x1FF2AF70 }
        else if cfg!(feature = "psp_395") { 0x63182EBF }
        else if cfg!(feature = "psp_380") { 0xFD347C2B }
        else { 0x29A5899B }
    )]
    pub unsafe fn sceKernelSetRebootKernel(
        reboot_kernel_fn: unsafe extern "C" fn(_: *mut c_void) -> i32,
    ) -> SceResult<()>;

    /// Calls the current set reboot kernel function.
    ///
    /// # Parameters
    ///
    /// - `args`: The arguments to pass to the function set with [`sceKernelSetRebootKernel`].
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xE73FBC0B }
        else if cfg!(feature = "psp_630") { 0xA51E2C33 }
        else if cfg!(feature = "psp_600") { 0xB0ED0CCD }
        else if cfg!(feature = "psp_570") { 0x1DDEFF98 }
        else if cfg!(feature = "psp_500") { 0xA9EDD556 }
        else if cfg!(feature = "psp_420") { 0x253141AD }
        else if cfg!(feature = "psp_395") { 0x9BE6AF16 }
        else if cfg!(feature = "psp_380") { 0x5782A1A2 }
        else { 0xF4390489 }
    )]
    pub unsafe fn sceKernelRebootKernel(args: *mut c_void) -> SceResult<()>;

    /// Register a get ID function, that will be used on [`sceKernelGetId`].
    ///
    /// # Parameters
    ///
    /// - `func`: The function pointer to register.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x310802A9 }
        else if cfg!(feature = "psp_630") { 0x25A760F0 }
        else if cfg!(feature = "psp_600") { 0x02AEA33F }
        else if cfg!(feature = "psp_570") { 0xE2FAB093 }
        else if cfg!(feature = "psp_500") { 0x4803B7F1 }
        else if cfg!(feature = "psp_420") { 0x80E8C038 }
        else if cfg!(feature = "psp_395") { 0x4CCDD642 }
        else if cfg!(feature = "psp_380") { 0xAB1F7C42 }
        else { 0x419DB8F4 }
    )]
    pub unsafe fn sceKernelRegisterGetIdFunc(
        func: unsafe extern "C" fn(path: *const c_char, id: *mut c_char) -> SceResult<i32>,
    ) -> SceResult<()>;

    /// Call the registered get ID function with [`sceKernelRegisterGetIdFunc`], call it, and then
    /// unregisters it.
    ///
    /// # Parameters
    ///
    /// - `path` **[[In parameter]]**: The path to pass to the registered function.
    /// - `id` **[[Out parameter]]**: A pointer to be passed to the registered function.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xD0C1460D }
        else if cfg!(feature = "psp_630") { 0x288B090A }
        else if cfg!(feature = "psp_600") { 0xC10C3C26 }
        else if cfg!(feature = "psp_570") { 0xD25644E1 }
        else if cfg!(feature = "psp_500") { 0x636CD5CF }
        else if cfg!(feature = "psp_420") { 0x667303DD }
        else if cfg!(feature = "psp_395") { 0xD7FCE447 }
        else if cfg!(feature = "psp_380") { 0x88021868 }
        else { 0xA1ACEA31 }
    )]
    pub unsafe fn sceKernelGetId(path: *const c_char, id: *mut c_char) -> SceResult<i32>;

    /// Makes the kernel to dump the internal memory table to kernel stdout.
    ///
    /// # Warning
    /// This functions in only available between the version 1.00 and 3.40 of the PSP firmware.
    #[nid(0x26F96157)]
    pub fn sceKernelSysMemDump();

    /// Makes the kernel to dump the list of memory blocks to kernel stdout.
    ///
    /// # Warning
    /// This functions in only available between the version 1.00 and 3.40 of the PSP firmware.
    #[nid(0x6D6200DD)]
    pub fn sceKernelSysMemDumpBlock();

    /// Makes the kernel to dump the tail blocks to kernel stdout.
    ///
    /// # Warning
    /// This functions in only available between the version 1.00 and 3.40 of the PSP firmware.
    #[nid(0x621037F5)]
    pub fn sceKernelSysMemDumpTail();
}

impl MemoryBlockId {
    /// Create a new memory block ID from a raw value.
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

    /// Create a new memory block ID structure from a raw value without checking value range.
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

impl HeapId {
    /// Create a new heap ID from a raw value.
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

    /// Create a new heap ID structure from a raw value without checking value range.
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

impl crate::private::Sealed for HeapId {}
unsafe impl SceResultOk for HeapId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe { SceUid::handle_ok_value(ok_value).map(Self) }
    }
}

impl crate::private::Sealed for HeapCreateFlag {}
unsafe impl SceResultOk for HeapCreateFlag {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        Ok(Self::from_bits_retain(ok_value))
    }
}

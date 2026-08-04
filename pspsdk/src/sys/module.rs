//! Module management.

use core::{ffi::c_void, num::NonZero};

use bitflag_attr::bitflag;
use pspsdk_macros::{psp_fw_cfg, psp_stub};

use crate::{
    allocators::MemoryPartitionId,
    sys::{
        io::FileId, mem::MemoryBlockId, thread::ThreadAttributes, SceError, SceIntoOkValue,
        SceResult, SceResultOk, SceSize, SceUid,
    },
};

/// The module UID.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ModuleId(SceUid);

/// Options of the module placement in memory on load time.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash, Default)]
pub enum LoadModulePosition {
    /// Loads module in the lowest possible address.
    #[default]
    Low  = 0,
    /// Loads module in the highest possible address.
    High = 1,
    /// Loads module in the specified address.
    Address = 2,
}

/// Options to configure the module loading behavior.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
#[doc(alias("SceKernelLMOption"))]
pub struct LoadModuleOptions {
    /// The size of this structure.
    pub size: SceSize,
    /// The memory partition where the module program will be loaded.
    pub text_partition: MemoryPartitionId,
    /// The memory partition where the module binary data will be loaded.
    pub data_partition: MemoryPartitionId,
    /// Unused.
    pub flags: u32,
    /// The memory placement policy to use during load time.
    pub position: LoadModulePosition,
    /// The load method to use. Unused.
    pub access: u8,
    pub reserved: [u8; 2],
}

/// Options to configure the module loading behavior.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
#[doc(alias("SceKernelSMOption"))]
pub struct StartModuleOptions {
    /// The size of this structure.
    pub size: SceSize,
    /// The memory partition where the module stack will be created.
    pub stack_partition: MemoryPartitionId,
    /// The size of the stack.
    pub stack_size: Option<NonZero<SceSize>>,
    /// The thread priority of the module thread.
    pub thread_priority: i32,
    /// The thread attribute of the module thread.
    pub thread_attribute: ThreadAttributes,
}

/// The attributes for modules.
///
/// Module attributes can be split into three categories:
/// - Mode flag: Attributes that dictates how the module behaves.
/// - Privilege flag: Attribute that set the permissions of the module.
/// - KIRK flag: Attribute related to KIRK encryption libraries used.
#[bitflag(u16)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum ModuleAttributes {
    /// Mode flag. The default mode.
    #[default]
    #[doc(alias("SCE_MODULE_ATTR_NONE"))]
    DefaultMode = 0x0000,
    /// Mode flag. The module stays in memory and cannot be unloaded.
    #[doc(alias("SCE_MODULE_ATTR_CANT_STOP"))]
    NoStopMode = 0x0001,
    /// Mode flag. Only one instance (version) of the module can be loaded into the system.
    ///
    /// If loading another version of that module is desired, it needs to delete the loaded
    /// version first.
    #[doc(alias("SCE_MODULE_ATTR_EXCLUSIVE_LOAD"))]
    ExclusiveLoadMode = 0x0002,
    /// Mode flag. Only one instance (version) of the module can be started.
    ///
    /// If starting another version of that module is desired, it needs to stop the running
    /// version first.
    #[doc(alias("SCE_MODULE_ATTR_EXCLUSIVE_START"))]
    ExclusiceStartMode = 0x0004,

    /// Privilege flag. User level module.
    #[doc(alias("SCE_MODULE_USER"))]
    User   = 0x0000,
    /// Privilege flag. Privilege level for Memory Stick modules (e.g. POPS/Demo).
    #[doc(alias("SCE_MODULE_MS"))]
    MemoryStick = 0x0200,
    /// Privilege flag. Privilege level for USB and WLAN modules (e.g. Gameshare).
    #[doc(alias("SCE_MODULE_USB_WLAN"))]
    UsbWlan = 0x0400,
    /// Privilege flag. Privilege level for Application modules (e.g. ComicReader/Skype).
    #[doc(alias("SCE_MODULE_APP"))]
    App    = 0x0600,
    /// Privilege flag. Privilege level for VSH/XMB modules.
    #[doc(alias("SCE_MODULE_VSH"))]
    Vsh    = 0x0800,
    /// Privilege flag. Privilege level for kernel modules.
    #[doc(alias("SCE_MODULE_KERNEL"))]
    Kernel = 0x1000,

    /// KIRK flag. No KIRK usage.
    NoKirk = 0x0000,
    /// KIRK flag. The module uses KIRK's `memlmd`` resident library.
    #[doc(alias("SCE_MODULE_KIRK_MEMLMD_LIB"))]
    MemlmdKirk = 0x2000,
    /// KIRK flag. The module uses KIRK's semaphore resident library.
    #[doc(alias("SCE_MODULE_KIRK_SEMAPHORE_LIB"))]
    SemaKirk = 0x4000,
}

/// Holds information about a module.
#[repr(C)]
#[derive(Debug, Clone)]
#[doc(alias("SceKernelModuleInfo"))]
pub struct KernelModuleInfo {
    /// The size of this structure.
    pub size: SceSize,
    /// The number of segments of the module
    pub num_segments: u8,
    /// Reserved or unused.
    pub reserved: [u8; 3],
    /// The start address of the segments of the module.
    pub segments_addr: [SceSize; 4],
    /// The size of the segments of the module.
    pub sefments_size: [SceSize; 4],
    /// The entry address of the module.
    pub entry_addr: SceSize,
    /// The value of the `gp` register.
    pub gp_value: u32,
    /// The start address of the text segment of the module.
    pub text_addr: SceSize,
    /// The size of the text segment of the module.
    pub text_size: SceSize,
    /// The size of the data segment of the module.
    pub data_size: SceSize,
    /// The size of the bss segment of the module.
    pub bss_size: SceSize,
    /// The module attributes.
    pub attribute: ModuleAttributes,
    /// The version of the library.
    ///
    /// It consists of a 'major' and 'minor' field.
    ///
    /// The version of a stub library shouldn't be higher than the version(s) of the corresponding
    /// resident library/libraries. Linking won't be performed in such a case.
    pub version: (u8, u8),
    /// The name of the library.
    pub name: [u8; 27],
    /// The string terminator.
    ///
    /// Always `b'\0'`
    pub string_terminator: u8,
}

/// Holds information about a module compatible with firmware version less or equal to v1.50.
#[repr(C)]
#[derive(Debug, Clone)]
#[doc(alias("SceKernelModuleInfoV1"))]
pub struct KernelModuleInfoV1 {
    /// The size of this structure.
    pub size: SceSize,
    /// The number of segments of the module
    pub num_segments: u8,
    /// Reserved or unused.
    pub reserved: [u8; 3],
    /// The start address of the segments of the module.
    pub segments_addr: [SceSize; 4],
    /// The size of the segments of the module.
    pub sefments_size: [SceSize; 4],
    /// The entry address of the module.
    pub entry_addr: SceSize,
    /// The value of the `gp` register.
    pub gp_value: u32,
    /// The start address of the text segment of the module.
    pub text_addr: SceSize,
    /// The size of the text segment of the module.
    pub text_size: SceSize,
    /// The size of the data segment of the module.
    pub data_size: SceSize,
    /// The size of the bss segment of the module.
    pub bss_size: SceSize,
}

#[psp_stub(libname = "ModuleMgrForUser", flags = 0x4009, use_crate)]
unsafe extern "C" {
    /// Loads a module.
    ///
    /// This function restricts where it can load from (such as from flash0) unless you call it in
    /// kernel mode.
    ///
    /// This function must be called from a thread.
    ///
    /// # Parameters
    ///
    /// - `path` **[[In parameter]]**: The path to the module.
    /// - `flags`: Unused, pass `0`.
    /// - `options` **[[In parameter]]**: The options configuring the module load behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns the loaded module UID on success, error value otherwise.
    #[nid(0x977DE386)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelLoadModule(
        path: *const u8, flags: u32, options: Option<&LoadModuleOptions>,
    ) -> SceResult<ModuleId>;

    /// Loads a module from the Memory Stick.
    ///
    /// This function restricts what it can load, e.g. it wont load plain executables.
    ///
    /// # Parameters
    ///
    /// - `path` **[[In parameter]]**: The path to the module.
    /// - `flags`: Unused, pass `0`.
    /// - `options` **[[In parameter]]**: The options configuring the module load behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns the loaded module UID on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.03.
    #[psp_fw_cfg(103..)]
    #[nid(0x710F61B5)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelLoadModuleMs(
        path: *const u8, flags: u32, options: Option<&LoadModuleOptions>,
    ) -> SceResult<ModuleId>;

    /// Loads a module from a file ID.
    ///
    /// This function must be called from a thread.
    ///
    /// # Parameters
    ///
    /// - `fd`: The module file ID.
    /// - `flags`: Unused, pass `0`.
    /// - `options` **[[In parameter]]**: The options configuring the module load behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns the loaded module UID on success, error value otherwise.
    #[nid(0xB7F46618)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelLoadModuleByID(
        fd: FileId, flags: u32, options: Option<&LoadModuleOptions>,
    ) -> SceResult<ModuleId>;

    /// Loads a module at a offset in a memory block.
    ///
    /// # Parameters
    /// - `path` **[[In parameter]]**: The path to the module.
    /// - `block_id`: The memory block UID for the module to be loaded.
    /// - `offset`: The offset in the memory block.
    ///
    /// # Return Value
    ///
    /// Returns the loaded module UID on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 3.50.
    #[psp_fw_cfg(350..)]
    #[nid(0xE4C4211C)]
    pub unsafe fn sceKernelLoadModuleWithBlockOffset(
        path: *const u8, block_id: MemoryBlockId, offset: u64,
    ) -> SceResult<ModuleId>;

    /// Loads a module at a offset in a memory block from a file ID.
    ///
    /// # Parameters
    /// - `fd`: The module file ID.
    /// - `block_id`: The memory block UID for the module to be loaded.
    /// - `offset`: The offset in the memory block.
    ///
    /// # Return Value
    ///
    /// Returns the loaded module UID on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 3.50.
    #[psp_fw_cfg(350..)]
    #[nid(0xFBE27467)]
    pub safe fn sceKernelLoadModuleByIDWithBlockOffset(
        fd: FileId, block_id: MemoryBlockId, offset: u64,
    ) -> SceResult<ModuleId>;

    /// Load a module from a buffer using the USB/WLAN API.
    ///
    /// Can only be called from kernel mode, or from a thread that has attributes of
    /// [`ThreadAttributes::UsbWlanMode`].
    ///
    /// # Parameters
    ///
    /// - `buf_size`: The size of the module file image buffer.
    /// - `buf` **[[In parameter]]**: The buffer with the module file image.
    /// - `flags`: Unused, pass `0`.
    /// - `options` **[[In parameter]]**: The options configuring the module load behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns the loaded module UID on success, error value otherwise.
    #[nid(0xF9275D98)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelLoadModuleBufferUsbWlan(
        buf_size: SceSize, buf: *const u8, flags: u32, options: Option<&LoadModuleOptions>,
    ) -> SceResult<ModuleId>;

    /// Starts as loaded module.
    ///
    /// # Parameters
    ///
    /// - `id`: The module UID from `LoadModule*` family of functions.
    /// - `arg_size`: The size of the `argp` in bytes. This will be passed to the module's
    ///   `module_start` routine.
    /// - `argp` **[[InOut parameter]]**: A pointer to the arguments to the module. This will be
    ///   passed to the module's `module_start` routine.
    /// - `mod_return` **[[Out parameter]]**: A reference to receive the return value of the
    ///   `module_start` routine.
    /// - `options` **[[In parameter]]**: The options configuring the module start behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns `ModuleId` zero value for modules that don't need to be made resident, or a module
    /// UID of the module that was started and made resident on success, error value otherwise.
    #[eabi(i5)]
    #[nid(0x50F0C1EC)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelStartModule(
        id: ModuleId, arg_size: SceSize, argp: *mut c_void, status: &mut i32,
        options: Option<&StartModuleOptions>,
    ) -> SceResult<ModuleId>;

    /// Stops a running module.
    ///
    /// # Parameters
    ///
    /// - `id`: The module UID from `LoadModule*` family of functions.
    /// - `arg_size`: The size of the `argp` in bytes. This will be passed to the module's
    ///   `module_stop` routine.
    /// - `argp` **[[InOut parameter]]**: A pointer to the arguments to the module. This will be
    ///   passed to the module's `module_stop` routine.
    /// - `mod_return` **[[Out parameter]]**: A reference to receive the return value of the
    ///   `module_stop` routine.
    /// - `options` **[[In parameter]]**: The options configuring the module stop behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns `ModuleId` zero value on success, or the module UID if stop entry was successful but
    /// the stop routine fails, error value otherwise.
    #[eabi(i5)]
    #[nid(0xD1FF982A)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelStopModule(
        id: ModuleId, arg_size: SceSize, argp: *mut c_void, status: &mut i32,
        options: Option<&StartModuleOptions>,
    ) -> SceResult<ModuleId>;

    /// Unloads a stopped module.
    ///
    /// # Parameters
    ///
    /// - `id`: The module UID from `LoadModule*` family of functions.
    ///
    /// # Return Value
    ///
    /// Returns the module UID of the unloaded module or zero, error value otherwise.
    #[nid(0x2E0911AA)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelUnloadModule(id: ModuleId) -> SceResult<ModuleId>;

    /// Stops and unloads the current module.
    ///
    /// # Parameters
    ///
    /// - `exit_code`: The exit code to use.
    /// - `arg_size`: The size of the `argp` in bytes. This will be passed to the module's
    ///   `module_stop` routine.
    /// - `argp` **[[InOut parameter]]**: A pointer to the arguments to the module. This will be
    ///   passed to the module's `module_stop` routine.
    ///
    /// # Return Value
    ///
    /// Returns an unknown value on success, error value otherwise.
    #[nid(0xD675EBB8)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelSelfStopUnloadModule(
        exit_code: u32, arg_size: SceSize, argp: *mut c_void,
    ) -> SceResult<i32>;

    /// Stops and unloads the current module.
    ///
    /// # Parameters
    ///
    /// - `arg_size`: The size of the `argp` in bytes. This will be passed to the module's
    ///   `module_stop` routine.
    /// - `argp` **[[InOut parameter]]**: A pointer to the arguments to the module. This will be
    ///   passed to the module's `module_stop` routine.
    /// - `mod_return` **[[Out parameter]]**: A reference to receive the return value of the
    ///   `module_stop` routine.
    /// - `options` **[[In parameter]]**: The options configuring the module stop behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[psp_fw_cfg(270..)]
    #[nid(0xCC1D3699)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelStopUnloadSelfModule(
        arg_size: SceSize, argp: *mut c_void, mod_return: &mut i32,
        options: Option<&StartModuleOptions>,
    ) -> SceResult<()>;

    /// Stops and unloads the current module with a exit code.
    ///
    /// # Parameters
    ///
    /// - `exit_code`: The exit code to use.
    /// - `arg_size`: The size of the `argp` in bytes. This will be passed to the module's
    ///   `module_stop` routine.
    /// - `argp` **[[InOut parameter]]**: A pointer to the arguments to the module. This will be
    ///   passed to the module's `module_stop` routine.
    /// - `mod_return` **[[Out parameter]]**: A reference to receive the return value of the
    ///   `module_stop` routine.
    /// - `options` **[[In parameter]]**: The options configuring the module stop behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[psp_fw_cfg(270..)]
    #[eabi(i5)]
    #[nid(0x8F2DF740)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelStopUnloadSelfModuleWithStatus(
        exit_code: u32, arg_size: SceSize, argp: *mut c_void, mod_return: &mut i32,
        options: Option<&StartModuleOptions>,
    ) -> SceResult<()>;

    /// Queries the information about module from its UID.
    ///
    /// **Note:** This fails on v1.00 firmware (and even it worked has a limited structure).
    ///
    /// # Parameters
    ///
    /// - `id`: The module UID of the module.
    /// - `info` **[[Out parameter]]**: A reference to the struct to receive the module information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(0x748CBED9)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelQueryModuleInfo(
        id: ModuleId, info: &mut KernelModuleInfo,
    ) -> SceResult<()>;

    /// Gets the module ID of the caller module.
    ///
    /// # Return Value
    ///
    /// Returns the loaded module UID on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[psp_fw_cfg(150..)]
    #[nid(0xF0A26395)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelGetModuleId() -> SceResult<ModuleId>;

    /// Get a list of module UIDs.
    ///
    /// # Parameters
    ///
    /// - `buf` **[[Out parameter]]**: A pointer to a buffer to receive the module ID list.
    /// - `buf_size`: The size of the buffer.
    /// - `id_count` **[[Out parameter]]**: A reference to receive the total number of loaded
    ///   modules.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[psp_fw_cfg(150..)]
    #[nid(0x644395E2)]
    #[cfg(not(feature = "kernel"))]
    pub unsafe fn sceKernelGetModuleIdList(
        buf: *mut ModuleId, buf_size: SceSize, id_cound: &mut SceSize,
    ) -> SceResult<()>;

    /// Gets the module ID of the module occupying the given address.
    ///
    /// # Parameters
    ///
    /// - `addr`: The address to query.
    ///
    /// # Return Value
    ///
    /// Returns the loaded module UID on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[psp_fw_cfg(150..)]
    #[nid(0xD8B73127)]
    #[cfg(not(feature = "kernel"))]
    pub safe fn sceKernelGetModuleIdByAddress(addr: SceSize) -> SceResult<ModuleId>;
}


// FIXME: Add missing functions
//
// - sceKernelLoadModuleForLoadExecForUser
// - Others
#[cfg(feature = "kernel")]
#[psp_stub(libname = "ModuleMgrForKernel", flags = 0x0009, use_crate)]
unsafe extern "C" {
    /// Loads a module.
    ///
    /// This function restricts where it can load from (such as from flash0) unless you call it in
    /// kernel mode.
    ///
    /// This function must be called from a thread.
    ///
    /// # Parameters
    ///
    /// - `path` **[[In parameter]]**: The path to the module.
    /// - `flags`: Unused, pass `0`.
    /// - `options` **[[In parameter]]**: The options configuring the module load behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns the loaded module UID on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x939E4270 }
        else if cfg!(feature = "psp_630") { 0xFFB9B760 }
        else if cfg!(feature = "psp_600") { 0xE3CCC6EA }
        else if cfg!(feature = "psp_570") { 0xDDCC9529 }
        else if cfg!(feature = "psp_500") { 0xC5A281C5 }
        else if cfg!(feature = "psp_420") { 0x5685394B }
        else if cfg!(feature = "psp_395") { 0x24C5ABC2 }
        else if cfg!(feature = "psp_380") { 0xAE802E8F }
        else { 0x977DE386 }
    )]
    pub unsafe fn sceKernelLoadModule(
        path: *const u8, flags: u32, options: Option<&LoadModuleOptions>,
    ) -> SceResult<ModuleId>;

    /// Loads a module from the Memory Stick.
    ///
    /// This function restricts what it can load, e.g. it wont load plain executables.
    ///
    /// # Parameters
    ///
    /// - `path` **[[In parameter]]**: The path to the module.
    /// - `flags`: Unused, pass `0`.
    /// - `options` **[[In parameter]]**: The options configuring the module load behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns the loaded module UID on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.00 and removed on version 1.51.
    #[nid(0x710F61B5)]
    #[psp_fw_cfg(..151)]
    pub unsafe fn sceKernelLoadModuleMs(
        path: *const u8, flags: u32, options: Option<&LoadModuleOptions>,
    ) -> SceResult<ModuleId>;

    /// Loads a module from a file ID.
    ///
    /// This function must be called from a thread.
    ///
    /// # Parameters
    ///
    /// - `fd`: The module file ID.
    /// - `flags`: Unused, pass `0`.
    /// - `options` **[[In parameter]]**: The options configuring the module load behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns the loaded module UID on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xEEC2A745 }
        else if cfg!(feature = "psp_630") { 0xA84CDD60 }
        else if cfg!(feature = "psp_600") { 0x3E9AA3D1 }
        else if cfg!(feature = "psp_570") { 0x1BBB3809 }
        else if cfg!(feature = "psp_500") { 0x087AE64D }
        else if cfg!(feature = "psp_420") { 0x15BDD18A }
        else if cfg!(feature = "psp_395") { 0xA79ED2B0 }
        else if cfg!(feature = "psp_380") { 0x25EDFE8C }
        else { 0xB7F46618 }
    )]
    pub safe fn sceKernelLoadModuleByID(
        fd: FileId, flags: u32, options: Option<&LoadModuleOptions>,
    ) -> SceResult<ModuleId>;

    /// Load a module from a buffer using the USB/WLAN API.
    ///
    /// Can only be called from kernel mode, or from a thread that has attributes of
    /// [`ThreadAttributes::UsbWlanMode`].
    ///
    /// # Parameters
    ///
    /// - `buf_size`: The size of the module file image buffer.
    /// - `buf` **[[In parameter]]**: The buffer with the module file image.
    /// - `flags`: Unused, pass `0`.
    /// - `options` **[[In parameter]]**: The options configuring the module load behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns the loaded module UID on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x2F3F9B6A }
        else if cfg!(feature = "psp_630") { 0x511DAF88 }
        else if cfg!(feature = "psp_600") { 0x5EBE54F1 }
        else if cfg!(feature = "psp_570") { 0x9976E1DB }
        else if cfg!(feature = "psp_500") { 0x92DDCE6A }
        else if cfg!(feature = "psp_420") { 0xE1D3CE25 }
        else if cfg!(feature = "psp_395") { 0xB25FFA4E }
        else if cfg!(feature = "psp_380") { 0x1F0F8DF2 }
        else if cfg!(feature = "psp_370") { 0xCE70664B }
        else { 0xF9275D98 }
    )]
    pub unsafe fn sceKernelLoadModuleBufferUsbWlan(
        buf_size: SceSize, buf: *const u8, flags: u32, options: Option<&LoadModuleOptions>,
    ) -> SceResult<ModuleId>;

    /// Starts as loaded module.
    ///
    /// # Parameters
    ///
    /// - `id`: The module UID from `LoadModule*` family of functions.
    /// - `arg_size`: The size of the `argp` in bytes. This will be passed to the module's
    ///   `module_start` routine.
    /// - `argp` **[[InOut parameter]]**: A pointer to the arguments to the module. This will be
    ///   passed to the module's `module_start` routine.
    /// - `mod_return` **[[Out parameter]]**: A reference to receive the return value of the
    ///   `module_start` routine.
    /// - `options` **[[In parameter]]**: The options configuring the module start behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns `ModuleId` zero value for modules that don't need to be made resident, or a module
    /// UID of the module that was started and made resident on success, error value otherwise.
    #[eabi(i5)]
    #[nid(if cfg!(feature = "psp_660") { 0x3FF74DF1 }
        else if cfg!(feature = "psp_630") { 0xE6BF3960 }
        else if cfg!(feature = "psp_600") { 0xDF8FFFAB }
        else if cfg!(feature = "psp_570") { 0xED45C046 }
        else if cfg!(feature = "psp_500") { 0xBB8C8FDF }
        else if cfg!(feature = "psp_420") { 0x63527600 }
        else if cfg!(feature = "psp_395") { 0x63A3CAFB }
        else if cfg!(feature = "psp_380") { 0x1D313DE9 }
        else { 0x50F0C1EC }
    )]
    pub unsafe fn sceKernelStartModule(
        id: ModuleId, arg_size: SceSize, argp: *mut c_void, mod_return: &mut i32,
        options: Option<&StartModuleOptions>,
    ) -> SceResult<ModuleId>;

    /// Stops a running module.
    ///
    /// # Parameters
    ///
    /// - `id`: The module UID from `LoadModule*` family of functions.
    /// - `arg_size`: The size of the `argp` in bytes. This will be passed to the module's
    ///   `module_stop` routine.
    /// - `argp` **[[InOut parameter]]**: A pointer to the arguments to the module. This will be
    ///   passed to the module's `module_stop` routine.
    /// - `mod_return` **[[Out parameter]]**: A reference to receive the return value of the
    ///   `module_stop` routine.
    /// - `options` **[[In parameter]]**: The options configuring the module stop behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns `ModuleId` zero value on success, or the module UID if stop entry was successful but
    /// the stop routine fails, error value otherwise.
    #[eabi(i5)]
    #[nid(if cfg!(feature = "psp_660") { 0xE5D6087B }
        else if cfg!(feature = "psp_630") { 0x4848E645 }
        else if cfg!(feature = "psp_600") { 0xAAFA90C2 }
        else if cfg!(feature = "psp_570") { 0x63440F98 }
        else if cfg!(feature = "psp_500") { 0xE0D3F771 }
        else if cfg!(feature = "psp_420") { 0x01F0C349 }
        else if cfg!(feature = "psp_395") { 0xEC942565 }
        else if cfg!(feature = "psp_380") { 0xAE4F1DCF }
        else { 0xD1FF982A }
    )]
    pub unsafe fn sceKernelStopModule(
        id: ModuleId, arg_size: SceSize, argp: *mut c_void, mod_return: &mut i32,
        options: Option<&StartModuleOptions>,
    ) -> SceResult<ModuleId>;

    /// Unloads a stopped module.
    ///
    /// # Parameters
    ///
    /// - `id`: The module UID from `LoadModule*` family of functions.
    ///
    /// # Return Value
    ///
    /// Returns the module UID of the unloaded module or zero, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x387E3CA9 }
        else if cfg!(feature = "psp_630") { 0x0D053026 }
        else if cfg!(feature = "psp_600") { 0x9CEB18C4 }
        else if cfg!(feature = "psp_570") { 0xCD5F5F22 }
        else if cfg!(feature = "psp_500") { 0x2AFF3E70 }
        else if cfg!(feature = "psp_420") { 0x55A7D7C8 }
        else if cfg!(feature = "psp_395") { 0xE02425E1 }
        else if cfg!(feature = "psp_380") { 0xB98CD891 }
        else { 0x2E0911AA }
    )]
    pub safe fn sceKernelUnloadModule(id: ModuleId) -> SceResult<ModuleId>;

    /// Stops and unloads the current module.
    ///
    /// # Parameters
    ///
    /// - `exit_code`: The exit code to use.
    /// - `arg_size`: The size of the `argp` in bytes. This will be passed to the module's
    ///   `module_stop` routine.
    /// - `argp` **[[InOut parameter]]**: A pointer to the arguments to the module. This will be
    ///   passed to the module's `module_stop` routine.
    ///
    /// # Return Value
    ///
    /// Returns an unknown value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x5805C1CA }
        else if cfg!(feature = "psp_630") { 0xD699583C }
        else if cfg!(feature = "psp_600") { 0x75ECB8BA }
        else if cfg!(feature = "psp_570") { 0x2128E1A6 }
        else if cfg!(feature = "psp_500") { 0x83088631 }
        else if cfg!(feature = "psp_420") { 0x08B3B762 }
        else if cfg!(feature = "psp_395") { 0xB1C431DF }
        else if cfg!(feature = "psp_380") { 0x121349C9 }
        else { 0xD675EBB8 }
    )]
    pub unsafe fn sceKernelSelfStopUnloadModule(
        exit_code: u32, arg_size: SceSize, argp: *mut c_void,
    ) -> SceResult<i32>;


    /// Stops and unloads the current module.
    ///
    /// # Parameters
    ///
    /// - `arg_size`: The size of the `argp` in bytes. This will be passed to the module's
    ///   `module_stop` routine.
    /// - `argp` **[[InOut parameter]]**: A pointer to the arguments to the module. This will be
    ///   passed to the module's `module_stop` routine.
    /// - `mod_return` **[[Out parameter]]**: A reference to receive the return value of the
    ///   `module_stop` routine.
    /// - `options` **[[In parameter]]**: The options configuring the module stop behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0xE97E0DB7 }
        else if cfg!(feature = "psp_630") { 0x291CF03E }
        else if cfg!(feature = "psp_600") { 0xB751A9DE }
        else if cfg!(feature = "psp_570") { 0x9652CC97 }
        else if cfg!(feature = "psp_500") { 0x48B58E0A }
        else if cfg!(feature = "psp_420") { 0x92B3EE4F }
        else if cfg!(feature = "psp_395") { 0xB66B7D9E }
        else if cfg!(feature = "psp_380") { 0x51512315 }
        else { 0xCC1D3699 }
    )]
    pub unsafe fn sceKernelStopUnloadSelfModule(
        arg_size: SceSize, argp: *mut c_void, mod_return: &mut i32,
        options: Option<&StartModuleOptions>,
    ) -> SceResult<()>;

    /// Stops and unloads the current module with a exit code.
    ///
    /// # Parameters
    ///
    /// - `exit_code`: The exit code to use.
    /// - `arg_size`: The size of the `argp` in bytes. This will be passed to the module's
    ///   `module_stop` routine.
    /// - `argp` **[[InOut parameter]]**: A pointer to the arguments to the module. This will be
    ///   passed to the module's `module_stop` routine.
    /// - `mod_return` **[[Out parameter]]**: A reference to receive the return value of the
    ///   `module_stop` routine.
    /// - `options` **[[In parameter]]**: The options configuring the module stop behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 2.70.
    #[psp_fw_cfg(270..)]
    #[eabi(i5)]
    #[nid(if cfg!(feature = "psp_660") { 0xEE6E8F49 }
        else if cfg!(feature = "psp_630") { 0x455DE851 }
        else if cfg!(feature = "psp_600") { 0xB9FAD518 }
        else if cfg!(feature = "psp_570") { 0x23DE5C86 }
        else if cfg!(feature = "psp_500") { 0xD7DDBD55 }
        else if cfg!(feature = "psp_420") { 0x914EFF06 }
        else if cfg!(feature = "psp_395") { 0xF89936FD }
        else if cfg!(feature = "psp_380") { 0x61129E61 }
        else { 0x8F2DF740 }
    )]
    pub unsafe fn sceKernelStopUnloadSelfModuleWithStatus(
        exit_code: u32, arg_size: SceSize, argp: *mut c_void, mod_return: &mut i32,
        options: Option<&StartModuleOptions>,
    ) -> SceResult<()>;

    /// Queries the information about module from its UID.
    ///
    /// **Note:** This fails on v1.00 firmware (and even it worked has a limited structure).
    ///
    /// # Parameters
    ///
    /// - `id`: The module UID of the module.
    /// - `info` **[[Out parameter]]**: A reference to the struct to receive the module information.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    #[nid(if cfg!(feature = "psp_660") { 0x22BDBEFF }
        else if cfg!(feature = "psp_630") { 0xEE3176DD }
        else if cfg!(feature = "psp_600") { 0x8F38EBE9 }
        else if cfg!(feature = "psp_570") { 0xD7B69988 }
        else if cfg!(feature = "psp_500") { 0xBE471B08 }
        else if cfg!(feature = "psp_420") { 0x1479AD37 }
        else if cfg!(feature = "psp_395") { 0x0A296830 }
        else if cfg!(feature = "psp_380") { 0x4729151D }
        else { 0x748CBED9 }
    )]
    pub safe fn sceKernelQueryModuleInfo(
        id: ModuleId, info: &mut KernelModuleInfo,
    ) -> SceResult<()>;

    /// Gets the module ID of the caller module.
    ///
    /// # Return Value
    ///
    /// Returns the loaded module UID on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[psp_fw_cfg(150..)]
    #[nid(if cfg!(feature = "psp_660") { 0xCAB06D30 }
        else if cfg!(feature = "psp_630") { 0x8B19C181 }
        else if cfg!(feature = "psp_600") { 0xB7CFD5AD }
        else if cfg!(feature = "psp_570") { 0x3DB107DB }
        else if cfg!(feature = "psp_500") { 0x9DC0EA88 }
        else if cfg!(feature = "psp_420") { 0x01A40221 }
        else if cfg!(feature = "psp_395") { 0xCECA0FFC }
        else if cfg!(feature = "psp_380") { 0x8FF98580 }
        else { 0xF0A26395 }
    )]
    pub safe fn sceKernelGetModuleId() -> SceResult<ModuleId>;

    /// Get a list of module UIDs.
    ///
    /// # Parameters
    ///
    /// - `buf` **[[Out parameter]]**: A pointer to a buffer to receive the module ID list.
    /// - `buf_size`: The size of the buffer.
    /// - `id_count` **[[Out parameter]]**: A reference to receive the total number of loaded
    ///   modules.
    ///
    /// # Return Value
    ///
    /// `Ok` value on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[psp_fw_cfg(150..)]
    #[nid(if cfg!(feature = "psp_660") { 0x303FAB7F }
        else if cfg!(feature = "psp_630") { 0xA95C26C8 }
        else if cfg!(feature = "psp_600") { 0x4392903C }
        else if cfg!(feature = "psp_570") { 0xEABD3D28 }
        else if cfg!(feature = "psp_500") { 0x30309A5D }
        else if cfg!(feature = "psp_420") { 0x8BB4C3D5 }
        else if cfg!(feature = "psp_395") { 0x4A61F528 }
        else if cfg!(feature = "psp_380") { 0x609FA413 }
        else { 0x644395E2 }
    )]
    pub unsafe fn sceKernelGetModuleIdList(
        buf: *mut ModuleId, buf_size: SceSize, id_cound: &mut SceSize,
    ) -> SceResult<()>;

    /// Gets the module ID of the module occupying the given address.
    ///
    /// # Parameters
    ///
    /// - `addr`: The address to query.
    ///
    /// # Return Value
    ///
    /// Returns the loaded module UID on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.50.
    #[psp_fw_cfg(150..)]
    #[nid(if cfg!(feature = "psp_660") { 0x433D5287 }
        else if cfg!(feature = "psp_630") { 0xAD633D71 }
        else if cfg!(feature = "psp_600") { 0x0F3F0B4C }
        else if cfg!(feature = "psp_570") { 0xDA59C81E }
        else if cfg!(feature = "psp_500") { 0xCBB6534C }
        else if cfg!(feature = "psp_420") { 0x6B2EB943 }
        else if cfg!(feature = "psp_395") { 0x0DFB5074 }
        else if cfg!(feature = "psp_380") { 0x5BE741EE }
        else { 0xD8B73127 }
    )]
    pub safe fn sceKernelGetModuleIdByAddress(addr: SceSize) -> SceResult<ModuleId>;


    /// Load module from a buffer with the Boot Init BTCNF apitype.
    ///
    /// # Parameters
    ///
    /// - `buf_size`: The size of the module file image buffer.
    /// - `buf` **[[In parameter]]**: The buffer with the module file image. The buffer must reside
    ///   at an address that is a multiple to `64` bytes.
    /// - `flags`: Unused, pass `0`.
    /// - `options` **[[In parameter]]**: The options configuring the module load behavior. If
    ///   [`None`], the function will assume default behavior.
    ///
    /// # Return Value
    ///
    /// Returns the loaded module UID on success, error value otherwise.
    ///
    /// # Firmware Version
    ///
    /// This API was introduced on PSP firmware version 1.52.
    #[psp_fw_cfg(152..)]
    #[nid(if cfg!(feature = "psp_660") { 0x1CF0B794 }
        else if cfg!(feature = "psp_630") { 0xB39242D0 }
        else if cfg!(feature = "psp_600") { 0x5226F362 }
        else if cfg!(feature = "psp_570") { 0xFEA867E4 }
        else if cfg!(feature = "psp_500") { 0xBCE510D3 }
        else if cfg!(feature = "psp_420") { 0x93CE0D0A }
        else if cfg!(feature = "psp_395") { 0x8EEB4576 }
        else if cfg!(feature = "psp_380") { 0xE2E2C6C6 }
        else { 0xEF7A7F02 }
    )]
    pub unsafe fn sceKernelLoadModuleBufferBootInitBtcnf(
        buf_size: SceSize, buf: *const u8, flags: u32, options: Option<&LoadModuleOptions>,
    ) -> SceResult<ModuleId>;
}


impl ModuleId {
    /// Create a new module ID from a raw value.
    ///
    /// This functions checks for the value of `raw` to be a in the range of possible `SceAtracId`
    /// values used by the PSP OS, returning an [`None`] otherwise.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        if let 0..=0x7FFFFFFF = raw {
            Some(unsafe { Self::from_raw_unchecked(raw) })
        } else {
            None
        }
    }

    /// Create a new module ID structure from a raw value without checking value range.
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
    pub const fn to_inner(self) -> u32 {
        // SAFETY: pattern types are always legal values of their base type
        // (Not using `.0` because that has perf regressions.)
        unsafe { core::mem::transmute(self) }
    }
}

impl crate::private::Sealed for ModuleId {}
unsafe impl SceResultOk for ModuleId {
    unsafe fn handle_ok_value(ok_value: u32) -> Result<Self, SceError> {
        unsafe { SceUid::handle_ok_value(ok_value).map(Self) }
    }
}
unsafe impl SceIntoOkValue for ModuleId {
    fn into_ok_value(self) -> u32 {
        self.to_inner()
    }
}

impl Default for LoadModuleOptions {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
            text_partition: Default::default(),
            data_partition: Default::default(),
            flags: Default::default(),
            position: Default::default(),
            access: Default::default(),
            reserved: Default::default(),
        }
    }
}

impl Default for StartModuleOptions {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
            stack_partition: Default::default(),
            stack_size: Default::default(),
            thread_priority: Default::default(),
            thread_attribute: Default::default(),
        }
    }
}

impl Default for KernelModuleInfo {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
            num_segments: Default::default(),
            reserved: Default::default(),
            segments_addr: Default::default(),
            sefments_size: Default::default(),
            entry_addr: Default::default(),
            gp_value: Default::default(),
            text_addr: Default::default(),
            text_size: Default::default(),
            data_size: Default::default(),
            bss_size: Default::default(),
            attribute: Default::default(),
            version: Default::default(),
            name: Default::default(),
            string_terminator: Default::default(),
        }
    }
}

impl Default for KernelModuleInfoV1 {
    fn default() -> Self {
        Self {
            size: size_of::<Self>(),
            num_segments: Default::default(),
            reserved: Default::default(),
            segments_addr: Default::default(),
            sefments_size: Default::default(),
            entry_addr: Default::default(),
            gp_value: Default::default(),
            text_addr: Default::default(),
            text_size: Default::default(),
            data_size: Default::default(),
            bss_size: Default::default(),
        }
    }
}

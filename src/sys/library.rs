//! Module for types related to libraries.

use core::ffi::c_void;

use bitflag_attr::bitflag;

use crate::sys::{LibFlags, SceSize};

pub const STUB_LIBRARY_ENTRY_TABLE_OLD_LEN: u8 = 6;
pub const STUB_LIBRARY_ENTRY_TABLE_NEW_LEN: u8 = 7;

pub const RESI_LIBRARY_ENTRY_TABLE_OLD_LEN: u8 = 4;
pub const RESI_LIBRARY_ENTRY_TABLE_NEW_LEN: u8 = 5;

/// Represents the imports, provided by a resident library, that a given module is using.
///
/// A module can have multiple stub libraries.
#[repr(C)]
#[doc(alias("SceStubLibraryEntryTable", "SceLibraryStubTable", "SceStubLibraryEntry"))]
pub struct StubLibraryEntry {
    /// The name of the library.
    pub name: *const u8,
    /// The version of the library.
    ///
    /// It consists of a 'major' and 'minor' field.
    ///
    /// The version of a stub library shouldn't be higher than the version(s) of the corresponding
    /// resident library/libraries. Linking won't be performed in such a case.
    pub version: (u8, u8),
    /// The library's flags.
    ///
    /// It can be set to either [`LibFlags::NoSpecialFlags`] or [`LibFlags::WeakImport`].
    pub flags: LibFlags,
    /// The length of the entry table in 32-Bit words.
    ///
    /// Set this to either [`STUB_LIBRARY_ENTRY_TABLE_OLD_LEN`] or
    /// [`STUB_LIBRARY_ENTRY_TABLE_NEW_LEN`].
    ///
    /// Use this member when you want to iterate through a list of entry tables `(size = len * 4)`.
    pub len: u8,
    /// The number of imported variables by the stub library.
    pub var_stub_count: u8,
    /// The number of imported functions by the stub library.
    pub func_stub_count: u16,
    /// A pointer to an array of NIDs containing the NIDs of the imported functions and variables.
    pub nid_table: *const u32,
    /// A pointer to an array of imported function stubs.
    pub func_stub_table: *const FunctionStub,
    /// A pointer to an array of imported variable stubs.
    pub var_stub_table: *const VariableStub,
    pub unk: u32,
}

unsafe impl Sync for StubLibraryEntry {}

#[repr(C)]
#[derive(Clone, Copy)]
#[doc(alias("SceStub"))]
pub union FunctionStub {
    pub direct_call: FunctionDirectCall,
    pub syscall: FunctionSyscall,
}

/// This type represents a function stub belonging to the same privilege-level linked libraries,
/// i.e. a kernel resident library linked with a kernel stub library.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionDirectCall {
    /// The call to the imported function via a MIPS ASM Jump instruction.
    pub call: SceSize,
    /// The delay slot belonging to the call, typically a NOP instruction.
    pub delay_slot: SceSize,
}

/// This type represents a function stub belonging to different privilege-level linked
/// libraries, i.e. a kernel resident library linked with a user stub library.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionSyscall {
    /// The return instruction from the stub. Typically a `JR $ra command.
    pub return_addr: SceSize,
    /// The system call exception used to call the imported function.
    pub syscall: SceSize,
}

/// This type represents an imported variable stub.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias("SceVariableStub"))]
pub struct VariableStub {
    /// The variable address.
    pub addr: SceSize,
    /// The variable NID, identifying the imported variable.
    pub nid: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union ResidentLibraryEntryItem {
    nid: u32,
    func: *const (),
    var: *mut c_void,
}

/// Represents a single resident export library of a module.
///
/// A module can have multiple resident libraries.
#[repr(C)]
#[doc(alias("SceResidentLibraryEntryTable", "SceLibraryEntryTable", "SceStubLibraryEntry"))]
pub struct ResidentLibraryEntry {
    /// The name of the library.
    pub name: *const u8,
    /// The version of the library.
    ///
    /// It consists of a 'major' and 'minor' field.
    ///
    /// The version of a resident library shouldn't be higher than the version(s) of the
    /// corresponding resident library/libraries. Linking won't be performed in such a case.
    pub version: (u8, u8),
    /// The library's flags.
    pub flags: LibFlags,
    /// The length of the entry table in 32-Bit words.
    ///
    /// Set this to either [`RESI_LIBRARY_ENTRY_TABLE_OLD_LEN`] or
    /// [`RESI_LIBRARY_ENTRY_TABLE_NEW_LEN`].
    ///
    /// Use this member when you want to iterate through a list of entry tables `(size = len * 4)`.
    pub len: u8,
    /// The number of exported variables by the resident library.
    pub var_exp_count: u8,
    /// The number of exported functions by the resident library.
    pub func_exp_count: u16,
    /// A pointer to the entry table that is an array composed of the NID entries followed by the
    /// address entries of the exported items.
    ///
    /// These arrays are used to correctly perform linking between a resident library and its
    /// corresponding stub libraries.
    pub entry_table: *mut ResidentLibraryEntryItem,
    pub unk1: u16,
    pub unk2: u8,
    pub unk3: u8,
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

/// The information of a module.
#[repr(C)]
#[derive(Debug, Clone)]
#[doc(alias("SceModuleInfo"))]
pub struct ModuleInfo {
    /// The attributes of a module.
    pub attributes: ModuleAttributes,
    /// The version of the module.
    ///
    /// It consists of a 'major' and 'minor' field.
    pub version: (u8, u8),
    /// The name of the module.
    pub name: [u8; 27],
    /// The string terminator.
    ///
    /// It is always `b'\0'`.
    pub terminal_char: u8,
    /// The global pointer of the module.
    pub gp: *mut c_void,
    /// A pointer to the first resident library entry table of the module.
    ///
    /// This section is known as ".lib.ent".
    pub entry_top: *mut c_void,
    /// A pointer to the last line of the ".lib.ent" section.
    ///
    /// This line is always `0` and it is known as ".lib.ent.btm".
    pub entry_end: *mut c_void,
    /// A pointer to the first stub library entry table of the module.
    ///
    /// This section is known as "lib.stub".
    pub stub_top: *mut c_void,
    /// A pointer to the last line of the "lib.stub" section.
    ///
    /// This line is always `0`` and is known as ".lib.stub.btm".
    pub stub_end: *mut c_void,
}

impl ModuleInfo {
    #[doc(hidden)]
    pub const fn name_from_str(s: &str) -> [u8; 27] {
        let bytes = s.as_bytes();
        let mut result = [0; 27];

        let mut i = 0;
        while i < bytes.len() {
            result[i] = bytes[i];

            i += 1;
        }

        result
    }
}

unsafe impl Sync for ModuleInfo {}

unsafe impl Sync for ResidentLibraryEntryItem {}
unsafe impl Sync for ResidentLibraryEntry {}

impl ResidentLibraryEntryItem {
    pub const fn new_nid(nid: u32) -> Self {
        Self { nid }
    }

    pub const fn new_fn(func: *const ()) -> Self {
        Self { func }
    }

    pub const fn new_var(var: *mut c_void) -> Self {
        Self { var }
    }
}

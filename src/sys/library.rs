//! Libraries information and types.

use core::ffi::c_void;

use crate::sys::{LibFlags, SceSize};

pub use crate::sys::module::ModuleAttributes;

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
    var: *const c_void,
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
    pub entry_table: *const ResidentLibraryEntryItem,
    pub unk1: u16,
    pub unk2: u8,
    pub unk3: u8,
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
    pub gp: *const u8,
    /// A pointer to the first resident library entry table of the module.
    ///
    /// This section is known as ".lib.ent".
    pub entry_top: *const u8,
    /// A pointer to the last line of the ".lib.ent" section.
    ///
    /// This line is always `0` and it is known as ".lib.ent.btm".
    pub entry_end: *const u8,
    /// A pointer to the first stub library entry table of the module.
    ///
    /// This section is known as "lib.stub".
    pub stub_top: *const u8,
    /// A pointer to the last line of the "lib.stub" section.
    ///
    /// This line is always `0`` and is known as ".lib.stub.btm".
    pub stub_end: *const u8,
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
unsafe impl Sync for StubLibraryEntry {}
unsafe impl Sync for ResidentLibraryEntryItem {}
unsafe impl Sync for ResidentLibraryEntry {}

impl ResidentLibraryEntryItem {
    pub const fn new_nid(nid: u32) -> Self {
        Self { nid }
    }

    pub const fn new_fn(func: *const ()) -> Self {
        Self { func }
    }

    pub const fn new_var(var: *const c_void) -> Self {
        Self { var }
    }
}

impl ModuleAttributes {
    #[inline]
    #[doc(hidden)]
    pub const fn base_default() -> Self {
        if cfg!(feature = "kernel") {
            Self::Kernel
        } else {
            Self::User
        }
    }
}

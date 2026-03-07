use bitflag_attr::bitflag;
use pspsdk_macros::psp_stub;

#[doc(hidden)]
pub mod macro_helpers;

pub mod library;

#[cfg(target_os = "psp")]
pub type SceSize = usize;
#[cfg(not(target_os = "psp"))]
pub type SceSize = u32;

#[cfg(target_os = "psp")]
pub type SceIsize = isize;
#[cfg(not(target_os = "psp"))]
pub type SceIsize = i32;

/// Resident/Stub library attributes.
///
/// Every library needs to have at least one of those attributes.
///
/// Resident libraries can have the members [`AutoExport`](SceLibFlags::AutoExport),
/// [`WeakExport`](SceLibFlags::WeakExport), [`NoLinkExport`](SceLibFlags::NoLinkExport),
/// [`SyscallExport`](SceLibFlags::SyscallExport) and [`IsSystemLib`](SceLibFlags::IsSystemLib).
///
/// Stub libraries can have [`NoSpecialFlags`](SceLibFlags::NoSpecialFlags) or
/// [`WeakImport`](SceLibFlags::WeakImport).
#[bitflag(u16)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum SceLibFlags {
    /// The library has no special attributes.
    NoSpecialFlags = 0x0,
    /// Automatically register the library to the system.
    AutoExport = 0x1,
    /// Indicates resident library can be overwritten.
    WeakExport = 0x2,
    /// Indicates resident library is NOT being linked.
    NoLinkExport = 0x4,
    /// Load module that references this library even if this library is not registered.
    WeakImport = 0x8,
    /// Indicates the use of the SYSCALL technique for linking.
    SyscallExport = 0x4000,
    /// The library is a system library (a mandatory library for all modules).
    IsSystemLib = 0x8000,
}

#[psp_stub(libname = "sceAtrac3plus", flags = 0x0009)]
extern "C" {
    #[nid(0x6A8C3CD5)]
    pub fn sceAtracDecodeData(
        atrac_id: i32, out_samples: *mut u16, out_n: *mut i32, out_end: *mut i32,
        out_remain_frame: *mut i32,
    ) -> i32;
}

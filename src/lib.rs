#![no_std]
#![allow(internal_features)]
#![feature(rustc_attrs, asm_experimental_arch, c_variadic, allocator_api)]
#![allow(improper_ctypes, reason = "Rust lint false positive (Rust issue #115457)")]

// Re-export proc-macros
pub use pspsdk_macros::{export, exports, psp_stub};


pub mod sys;

#[cfg(target_os = "psp")]
#[cfg(feature = "non-stub-code")]
pub mod alloc;

#[doc(hidden)]
pub mod eabi;

mod private {
    pub trait Sealed {}

    impl Sealed for () {}
    impl Sealed for bool {}
    impl Sealed for u8 {}
    impl Sealed for i8 {}
    impl Sealed for u16 {}
    impl Sealed for i16 {}
    impl Sealed for u32 {}
    impl Sealed for i32 {}
    impl Sealed for usize {}
    impl Sealed for isize {}
    impl<T> Sealed for *const T {}
    impl<T> Sealed for *mut T {}
    impl<T> Sealed for &T {}
    impl<T> Sealed for &mut T {}
}


// Setup things
#[cfg(all(target_os = "psp", feature = "non-stub-code"))]
core::arch::global_asm!(
    r#"
        .section .lib.ent.top, "a", @progbits
        .align 2
        .word 0
    .global __lib_ent_top
    __lib_ent_top:
        .section .lib.ent.btm, "a", @progbits
        .align 2
    .global __lib_ent_bottom
    __lib_ent_bottom:
        .word 0

        .section .lib.stub.top, "a", @progbits
        .align 2
        .word 0
    .global __lib_stub_top
    __lib_stub_top:
        .section .lib.stub.btm, "a", @progbits
        .align 2
    .global __lib_stub_bottom
    __lib_stub_bottom:
        .word 0
    "#
);
/// Declare a PSP module info.
///
/// This **does not** include basic `syslib`` export like [`module`].
#[macro_export]
macro_rules! module_info {
    ($name:expr, $version_major:expr, $version_minor:expr) => {
        #[used]
        #[unsafe(no_mangle)]
        #[unsafe(link_section = ".rodata.sceModuleInfo")]
        static module_info: $crate::Align16<$crate::sys::library::ModuleInfo> =
            $crate::Align16($crate::sys::library::ModuleInfo {
                attributes: $crate::sys::library::ModuleAttributes::from_bits_retain(0),
                version: ($version_major, $version_minor),
                name: $crate::sys::library::ModuleInfo::name_from_str($name),
                terminal_char: b'\0',
                gp: unsafe { (&raw const _gp).cast_mut().cast() },
                stub_top: unsafe { (&raw const __lib_stub_top).cast_mut().cast() },
                stub_end: unsafe { (&raw const __lib_stub_bottom).cast_mut().cast() },
                entry_top: unsafe { (&raw const __lib_ent_top).cast_mut().cast() },
                entry_end: unsafe { (&raw const __lib_ent_bottom).cast_mut().cast() },
            });

        unsafe extern "C" {
            static _gp: u8;
            static __lib_ent_bottom: u8;
            static __lib_ent_top: u8;
            static __lib_stub_bottom: u8;
            static __lib_stub_top: u8;
        }
    };
}


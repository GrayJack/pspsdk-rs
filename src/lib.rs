#![no_std]
#![allow(internal_features)]
#![feature(rustc_attrs, asm_experimental_arch, c_variadic, allocator_api)]
#![allow(improper_ctypes, reason = "Rust lint false positive (Rust issue #115457)")]

pub mod sys;

#[cfg(target_os = "psp")]
#[cfg(not(feature = "stub-only"))]
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
#[cfg(all(target_os = "psp", not(feature = "stub-only")))]
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

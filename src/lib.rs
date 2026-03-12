#![no_std]
#![allow(internal_features)]
#![feature(rustc_attrs, asm_experimental_arch, c_variadic)]
#![allow(improper_ctypes, reason = "Rust lint false positive (Rust issue #115457)")]

pub mod sys;

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

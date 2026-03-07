#![no_std]
#![allow(internal_features)]
#![feature(rustc_attrs)]

pub mod sys;


mod private {
    pub trait Sealed {}

    impl Sealed for () {}
    impl<T> Sealed for *const T {}
    impl<T> Sealed for *mut T {}
    impl<T> Sealed for &T {}
    impl<T> Sealed for &mut T {}
}

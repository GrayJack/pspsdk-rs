#![no_std]
#![allow(internal_features)]
#![allow(improper_ctypes, reason = "Rust lint false positive (Rust issue #115457)")]
#![feature(
    rustc_attrs,
    pattern_types,
    pattern_type_macro,
    pattern_type_range_trait,
    structural_match,
    asm_experimental_arch,
    c_variadic,
    allocator_api,
    alloc_error_handler,
    std_internals,
    core_intrinsics,
    lang_items,
    negative_impls,
    sync_unsafe_cell
)]
#![cfg_attr(feature = "non-stub-code", feature(panic_unwind))]
#![cfg_attr(feature = "std", feature(psp_std))]

#[cfg(feature = "non-stub-code")]
extern crate alloc;
#[cfg(feature = "non-stub-code")]
extern crate panic_unwind;
#[cfg(all(feature = "std", feature = "non-stub-code"))]
extern crate std;

// Re-export proc-macros
pub use pspsdk_macros::{export, exports, psp_stub};

use crate::sys::{thread::CallbackTermState, SceResult};

pub mod sys;

#[cfg(feature = "non-stub-code")]
pub mod allocators;
#[cfg(feature = "non-stub-code")]
pub mod panic;

#[cfg(feature = "non-stub-code")]
pub mod sync;

#[cfg(feature = "non-stub-code")]
pub mod io;

#[doc(hidden)]
pub mod eabi;

mod macros;

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

/// Sets the PSP OS functions for the [`io`] module when `non-stub-code` is enabled.
///
/// When `non-stub-code` is set, this is no-op
#[inline]
#[doc(hidden)]
pub fn set_psp_os_functions() {
    cfg_select! {
        feature = "non-stub-code" => {
            io_core::os::set_os_functions(io::PSP_OS_FUNCS);
        }
        _ => {}
    }
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

#[doc(hidden)]
#[repr(align(16))]
#[derive(Copy, Clone)]
pub struct Align16<T>(pub T);

#[cfg(feature = "std")]
unsafe extern "C" {
    #[link_name = "main"]
    #[doc(hidden)]
    pub fn c_main(argc: isize, argv: *const *const u8) -> isize;
}

/// Enable the home button.
///
/// This API does not have destructor support yet. You can manually setup an
/// exit callback if you need this, see the source code of this function.
pub fn enable_home_button() {
    use core::{ffi::c_void, ptr};
    use sys::thread::ThreadAttributes;

    unsafe {
        unsafe extern "C" fn exit_thread(_args: usize, _argp: *mut c_void) -> SceResult<u32> {
            unsafe extern "C" fn exit_callback(
                _arg1: u32, _arg2: i32, _arg: *mut c_void,
            ) -> CallbackTermState {
                let _res = sys::loadexec::sceKernelExitGame();
                CallbackTermState::NormalTermination
            }

            let res = unsafe {
                sys::thread::sceKernelCreateCallback(
                    c"exit_callback".as_ptr().cast(),
                    exit_callback,
                    ptr::null_mut(),
                )
            };

            let raw_res = res.as_inner();
            let Ok(id) = res.into_result() else {
                return SceResult::new(raw_res);
            };

            let _res = sys::loadexec::sceKernelRegisterExitCallback(id);
            let _res = sys::thread::sceKernelSleepThreadCB();

            SceResult::new(0)
        }

        // Enable the home button.
        let id = sys::thread::sceKernelCreateThread(
            &b"exit_thread\0"[0],
            exit_thread,
            32,
            0x1000,
            ThreadAttributes::empty(),
            None,
        )
        .into_result();

        let Ok(id) = id else {
            return;
        };

        let _res = sys::thread::sceKernelStartThread(id, 0, ptr::null_mut());
    }
}

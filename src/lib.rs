#![no_std]
#![allow(internal_features)]
#![allow(improper_ctypes, reason = "Rust lint false positive (Rust issue #115457)")]
#![feature(
    rustc_attrs,
    pattern_types,
    pattern_type_macro,
    structural_match,
    asm_experimental_arch,
    c_variadic,
    allocator_api,
    alloc_error_handler,
    std_internals,
    core_intrinsics,
    lang_items,
    negative_impls
)]
#![cfg_attr(feature = "non-stub-code", feature(panic_unwind))]
// #![cfg_attr(feature = "std", feature(psp_std))]

#[cfg(feature = "non-stub-code")]
extern crate alloc;
#[cfg(feature = "non-stub-code")]
extern crate panic_unwind;
#[cfg(all(feature = "std", feature = "non-stub-code", not(target_os = "psp")))]
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

#[cfg(feature = "non-stub-code")]
pub mod os;

#[cfg(feature = "non-stub-code")]
pub mod time;

#[doc(hidden)]
pub mod eabi;

mod macros;

mod private {
    #[cfg(feature = "non-stub-code")]
    use crate::sys::SceSize;

    pub trait Sealed {}

    impl Sealed for () {}
    impl Sealed for bool {}
    impl Sealed for u8 {}
    impl Sealed for i8 {}
    impl Sealed for u16 {}
    impl Sealed for i16 {}
    impl Sealed for u32 {}
    impl Sealed for i32 {}
    impl Sealed for u64 {}
    impl Sealed for i64 {}
    impl Sealed for usize {}
    impl Sealed for isize {}
    impl<T> Sealed for *const T {}
    impl<T> Sealed for *mut T {}
    impl<T> Sealed for &T {}
    impl<T> Sealed for &mut T {}

    // Libc functions that we need for rust linker

    #[unsafe(no_mangle)]
    #[cfg(feature = "non-stub-code")]
    unsafe extern "C" fn memset(ptr: *mut u8, value: u32, num: SceSize) -> *mut u8 {
        unsafe {
            cfg_select! {
                feature = "kernel" => crate::sys::libc::memset(ptr, value as i32, num),
                all(not(feature = "kernel"), feature = "cfw-api") => crate::sys::libc::memset(ptr, value as i32, num) ,
                _ => {
                    let mut i = 0;

                    while i < num {
                        *((ptr as SceSize + i) as *mut u8) = value as u8;
                        i += 1;
                    }

                    ptr
                }
            }
        }
    }

    #[unsafe(no_mangle)]
    #[cfg(feature = "non-stub-code")]
    unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, num: SceSize) -> *mut u8 {
        unsafe {
            cfg_select! {
                feature = "kernel" => crate::sys::libc::memcpy(dst, src, num),
                all(not(feature = "kernel"), feature = "cfw-api") => crate::sys::libc::memcpy(dst, src, num),
                _ => {
                    let mut i = 0;

                    while i < num {
                        *((dst as SceSize + i) as *mut u8) = *((src as SceSize + i) as *mut u8);
                        i += 1;
                    }

                    dst
                }
            }
        }
    }

    #[unsafe(no_mangle)]
    #[cfg(feature = "non-stub-code")]
    unsafe extern "C" fn memcmp(ptr1: *mut u8, ptr2: *mut u8, num: SceSize) -> i32 {
        unsafe {
            cfg_select! {
                feature = "kernel" => crate::sys::libc::memcmp(ptr1, ptr2, num),
                all(not(feature = "kernel"), feature = "cfw-api") => crate::sys::libc::memcmp(ptr1, ptr2, num),
                _ => {
                    let mut i = 0;

                    while i < num {
                        let val1 = *((ptr1 as SceSize + i) as *mut u8);
                        let val2 = *((ptr2 as SceSize + i) as *mut u8);
                        let diff = val1 as i32 - val2 as i32;

                        if diff != 0 {
                            return diff;
                        }

                        i += 1;
                    }

                    0
                }
            }
        }
    }

    #[unsafe(no_mangle)]
    #[cfg(feature = "non-stub-code")]
    unsafe extern "C" fn memmove(dst: *mut u8, src: *mut u8, num: SceSize) -> *mut u8 {
        unsafe {
            cfg_select! {
                feature = "kernel" => crate::sys::libc::memmove(dst, src, num),
                all(not(feature = "kernel"), feature = "cfw-api") => crate::sys::libc::memmove(dst, src, num),
                _ => {
                    if dst < src {
                        let mut i = 0;

                        while i < num {
                            *((dst as SceSize + i) as *mut u8) = *((src as SceSize + i) as *mut u8);
                            i += 1;
                        }
                    } else {
                        let mut i = num - 1;

                        while i > 0 {
                            *((dst as SceSize + i) as *mut u8) = *((src as SceSize + i) as *mut u8);
                            i -= 1;
                        }
                    }

                    dst
                }
            }
        }
    }

    #[unsafe(no_mangle)]
    #[cfg(feature = "non-stub-code")]
    unsafe extern "C" fn strlen(s: *mut u8) -> SceSize {
        unsafe {
            cfg_select! {
                feature = "kernel" => crate::sys::libc::strlen(s),
                all(not(feature = "kernel"), feature = "cfw-api") => crate::sys::libc::strlen(s),
                _ => {
                    let mut len = 0;

                    while *s.wrapping_add(len) != 0 {
                        len += 1;
                    }

                    len as SceSize
                }
            }
        }
    }
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
                _arg1: u32, _arg2: u32, _arg: *mut c_void,
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

#![no_std]
#![allow(internal_features)]
#![allow(improper_ctypes, reason = "Rust lint false positive (Rust issue #115457)")]
#![allow(unused_features, reason = "behavior changes in compilation context")]
#![feature(
    rustc_attrs,
    pattern_types,
    pattern_type_macro,
    structural_match,
    asm_experimental_arch,
    allocator_api,
    alloc_error_handler,
    std_internals,
    core_intrinsics,
    lang_items,
    negative_impls,
    try_trait_v2,
    try_trait_v2_residual,
    never_type,
    sync_unsafe_cell,
    slice_ptr_get
)]
#![cfg_attr(
    all(feature = "non-stub-code", not(panic = "immediate-abort")),
    feature(panic_unwind)
)]
#![cfg_attr(doc, feature(doc_cfg))]
// docs.rs needs this
#![cfg_attr(doc, feature(c_variadic))]
// #![cfg_attr(feature = "std", feature(psp_std))]

#[cfg(feature = "non-stub-code")]
extern crate alloc;
#[cfg(all(feature = "non-stub-code", not(panic = "immediate-abort")))]
extern crate panic_unwind;
#[cfg(all(feature = "std", feature = "non-stub-code", not(target_os = "psp")))]
extern crate std;

// Re-export proc-macros
pub use pspsdk_macros::{export, exports, psp_stub};

use crate::sys::{thread::CallbackTermState, SceResult};

pub mod sys;

// Custom modules to provide higher level API to things outside of rust STD
#[cfg(feature = "non-stub-code")]
pub mod allocators;
#[cfg(feature = "non-stub-code")]
pub mod power;

// Mixed modules (part STD-like and part custom).
#[cfg(feature = "non-stub-code")]
pub mod process;

// STD-like modules
#[cfg(feature = "non-stub-code")]
pub mod io;
#[cfg(feature = "non-stub-code")]
pub mod os;
#[cfg(feature = "non-stub-code")]
#[path = "panic.rs"]
pub mod panicking;
#[cfg(feature = "non-stub-code")]
pub mod sync;
#[cfg(feature = "non-stub-code")]
pub mod time;

#[cfg(feature = "non-stub-code")]
mod rt;
#[cfg(all(feature = "non-stub-code", not(feature = "std")))]
pub use rt::{init_cwd, module_start_init, process_argc_argv, psp_start, set_custom_cleanup};

#[doc(hidden)]
pub mod eabi;

mod macros;

mod private {
    #[cfg(feature = "non-stub-code")]
    mod string_impl;
    #[cfg(feature = "non-stub-code")]
    use core::ffi::{c_char, c_void};

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
    impl Sealed for core::convert::Infallible {}
    impl Sealed for ! {}

    /// Types that can work on volatile operation.
    pub trait VolatileOpAllowed: Sealed {}
    impl VolatileOpAllowed for bool {}
    impl VolatileOpAllowed for u8 {}
    impl VolatileOpAllowed for i8 {}
    impl VolatileOpAllowed for u16 {}
    impl VolatileOpAllowed for i16 {}
    impl VolatileOpAllowed for u32 {}
    impl VolatileOpAllowed for i32 {}
    impl VolatileOpAllowed for u64 {}
    impl VolatileOpAllowed for i64 {}
    impl VolatileOpAllowed for usize {}
    impl VolatileOpAllowed for isize {}

    // Libc functions that we need for rust linker

    #[unsafe(no_mangle)]
    #[cfg(feature = "non-stub-code")]
    unsafe extern "C" fn memset(src: *mut c_void, value: i32, num: SceSize) -> *mut c_void {
        unsafe {
            cfg_select! {
                feature = "kernel" => crate::sys::libc::memset(src.cast(), value, num).cast(),
                all(not(feature = "kernel"), not(feature = "cfw-api"), feature = "use-stub-c") => {
                    crate::sys::usersystemlib::sceKernelMemset(src.cast(), value, num).cast()
                },
                all(not(feature = "kernel"), feature = "cfw-api", feature = "use-stub-c") => {
                    crate::sys::libc::memset(src, value as i32, num)
                },
                _ => {
                    string_impl::set_bytes(src.cast(), value as u8, num);
                    src
                }
            }
        }
    }

    #[unsafe(no_mangle)]
    #[cfg(feature = "non-stub-code")]
    unsafe extern "C" fn memcpy(dst: *mut c_void, src: *const c_void, num: SceSize) -> *mut c_void {
        unsafe {
            cfg_select! {
                feature = "kernel" => crate::sys::libc::memcpy(dst.cast(), src.cast(), num).cast(),
                all(not(feature = "kernel"), feature = "use-stub-c") => crate::sys::usersystemlib::sceKernelMemcpy(dst.cast(), src.cast(), num).cast(),
                all(not(feature = "kernel"), feature = "cfw-api", feature = "use-stub-c") => {
                    crate::sys::libc::memcpy(dst, src, num)
                },
                _ => {
                    string_impl::copy_forward(dst.cast(), src.cast(), num);
                    dst
                }
            }
        }
    }

    #[unsafe(no_mangle)]
    #[cfg(feature = "non-stub-code")]
    unsafe extern "C" fn memcmp(ptr1: *const c_void, ptr2: *const c_void, num: SceSize) -> i32 {
        unsafe {
            cfg_select! {
                feature = "kernel" => crate::sys::libc::memcmp(ptr1.cast(), ptr2.cast(), num),
                all(not(feature = "kernel"), feature = "cfw-api", feature = "use-stub-c") => crate::sys::libc::memcmp(ptr1.cast(), ptr2.cast(), num),
                _ => {
                    string_impl::compare_bytes(ptr1.cast(), ptr2.cast(), num)
                }
            }
        }
    }

    #[unsafe(no_mangle)]
    #[cfg(feature = "non-stub-code")]
    unsafe extern "C" fn memmove(
        dst: *mut c_void, src: *const c_void, num: SceSize,
    ) -> *mut c_void {
        unsafe {
            cfg_select! {
                feature = "kernel" => crate::sys::libc::memmove(dst.cast(), src.cast(), num).cast(),
                all(not(feature = "kernel"), feature = "cfw-api", feature = "use-stub-c") => crate::sys::libc::memmove(dst, src, num),
                _ => {
                    use string_impl::PointersOverlap;

                    let src_byte = src as *const u8;
                    let dest_byte = dst as *mut u8;

                    // If the pointer overlap, we have to copy backwards
                    if src_byte.overlaps_with(dest_byte, num as _) {
                        string_impl::copy_backward(dest_byte, src_byte, num as _);
                    } else {
                        string_impl::copy_forward(dest_byte, src_byte, num as _);
                    }

                    dst
                }
            }
        }
    }

    #[unsafe(no_mangle)]
    #[cfg(feature = "non-stub-code")]
    pub(crate) unsafe extern "C" fn strlen(s: *const c_char) -> SceSize {
        unsafe {
            cfg_select! {
                feature = "kernel" => crate::sys::libc::strlen(s.cast()),
                all(not(feature = "kernel"), feature = "cfw-api", feature = "use-stub-c") => crate::sys::libc::strlen(s),
                _ => {
                    string_impl::c_string_length(s.cast())
                }
            }
        }
    }
}

/// Sets the PSP OS functions for the [`io`] module when `non-stub-code` is enabled.
///
/// When `non-stub-code` is set, this is no-op
#[inline]
#[cfg(feature = "non-stub-code")]
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
#[cfg(feature = "non-stub-code")]
pub fn enable_home_button() {
    use core::{ffi::c_void, ptr};
    use sys::thread::ThreadAttributes;

    unsafe {
        unsafe extern "C" fn exit_thread(_args: usize, _argp: *mut c_void) -> SceResult<u32> {
            #[allow(unreachable_code)]
            unsafe extern "C" fn exit_callback(
                _arg1: u32, _arg2: u32, _arg: *mut c_void,
            ) -> CallbackTermState {
                crate::rt::cleanup();
                process::exit_main(0);
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

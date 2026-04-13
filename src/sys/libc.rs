//! Module for libc functions
//!
//! PSP has many sources for those functions:
//!
//! - `SysclibForKernel` - kernel only
//! - `scePaf` - VSH/XMB only
//! - `SysclibForUser` - CFW lib
//! - `PSPSDK` - static lib, here they are substituted to a Rust implementation
//!
//! Whenever possible, using one of the first three is ideal to reuse the implementation and reduce
//! the software size and memory usage.
//!
//! > NOTE: `SysclibForUser` export those functions as syscalls, that means they are slightly less
//! > performant than `SysclibForKernel` and `scePaf` in the user-level context.
#![allow(unused_imports)]

use pspsdk_macros::psp_stub;

use crate::sys::{SceIsize, SceSize};

// FIXME: Missing funcs
// - (look_ctype_table, 0x32C767F2)
// - (prnt, 0x87C78FB6)
#[cfg(feature = "kernel")]
#[psp_stub(libname = "SysclibForKernel", flags = 0x0001, version = (0x00, 0x11))]
extern "C" {
    #[nid(0x7F8A6F23)]
    unsafe fn bcmp(source: *const u8, other: *const u8, len: SceSize) -> i32;

    #[nid(0x097049BD)]
    unsafe fn bcopy(src: *const u8, dest: *mut u8, len: SceSize);

    #[nid(0x86FEFCE9)]
    unsafe fn bzero(str: *mut u8, len: SceSize);

    #[nid(0xD1CD40E5)]
    unsafe fn index(str: *const u8, ch: i32) -> *mut u8;

    #[nid(0x243665ED)]
    unsafe fn rindex(str: *const u8, ch: i32) -> *mut u8;

    #[nid(0x68A78817)]
    unsafe fn memchr(src: *const u8, byte: i32, len: SceSize) -> *mut u8;

    #[nid(0x81D0D1F7)]
    unsafe fn memcmp(src: *mut u8, other: *mut u8, len: SceSize) -> i32;

    #[nid(0xAB7592FF)]
    unsafe fn memcpy(dest: *mut u8, src: *const u8, len: SceSize) -> *mut u8;

    #[nid(0xA48D2592)]
    unsafe fn memmove(dest: *mut u8, src: *const u8, len: SceSize) -> *mut u8;

    #[nid(0x10F3BB61)]
    unsafe fn memset(src: *mut u8, val: i32, len: SceSize) -> *mut u8;

    #[nid(0xC2145E80)]
    unsafe fn snprintf(str: *mut u8, len: SceSize, format: *const u8, ...) -> i32;

    #[nid(0x7661E728)]
    unsafe fn sprintf(str: *mut u8, format: *const u8, ...) -> i32;

    #[nid(0x476FD94A)]
    unsafe fn strcat(dest: *mut u8, src: *const u8) -> *mut u8;

    #[nid(0xB1DC2AE8)]
    unsafe fn strchr(str: *const u8, ch: i32) -> *mut u8;

    #[nid(0xC0AB8932)]
    unsafe fn strcmp(str1: *const u8, str2: *const u8) -> i32;

    #[nid(0xEC6F1CF2)]
    unsafe fn strcpy(dest: *mut u8, src: *const u8) -> *mut u8;

    #[nid(0x52DF196C)]
    unsafe fn strlen(str: *const u8) -> SceSize;

    #[nid(0x7AB35214)]
    unsafe fn strncmp(str1: *const u8, str2: *const u8, max_size: SceSize) -> i32;

    #[nid(0xB49A7697)]
    unsafe fn strncpy(dest: *mut u8, src: *const u8, max_size: SceSize) -> *mut u8;

    #[nid(0x90C5573D)]
    unsafe fn strnlen(str: *const u8, max_size: SceSize) -> SceSize;

    #[nid(0x0DFB7B6C)]
    unsafe fn strpbrk(str: *const u8, break_set: *const u8) -> *mut u8;

    #[nid(0x4C0E0274)]
    unsafe fn strrchr(str: *const u8, ch: i32) -> *mut u8;

    #[nid(0x0D188658)]
    unsafe fn strstr(str: *const u8, pattern: *const u8) -> *mut u8;

    #[nid(0x47DD934D)]
    unsafe fn strtol(str: *const u8, end_ptr: *mut *mut u8, base: i32) -> i32;

    #[nid(0x6A7900E1)]
    unsafe fn strtoul(str: *const u8, end_ptr: *mut *mut u8, base: i32) -> u32;

    #[nid(0x3EC5BBF6)]
    fn tolower(ch: i32) -> i32;

    #[nid(0xCE2F7487)]
    fn toupper(ch: i32) -> i32;

    #[nid(0x1493EBD9)]
    unsafe fn wmemset(str: *mut u16, val: u16, len: SceSize) -> *mut u16;

    #[nid(0x7DEE14DE)]
    fn __udivdi3(a: u32, b: u32) -> u32;

    #[nid(0xDF17F4A2)]
    unsafe fn __udivmoddi4(a: u32, b: u32, c: *mut u32) -> u32;

    #[nid(0x5E8E5F42)]
    fn __umoddi3(a: u32, b: u32) -> u32;
}

#[cfg(all(not(feature = "kernel"), feature = "cfw-api"))]
#[psp_stub(libname = "SysclibForUser", flags = 0x4001, version = (0x00, 0x00))]
extern "C" {
    #[nid(0x7F8A6F23)]
    unsafe fn bcmp(source: *const u8, other: *const u8, len: SceSize) -> i32;

    #[nid(0x097049BD)]
    unsafe fn bcopy(src: *const u8, dest: *mut u8, len: SceSize);

    #[nid(0x86FEFCE9)]
    unsafe fn bzero(str: *mut u8, len: SceSize);

    #[nid(0xD1CD40E5)]
    unsafe fn index(str: *const u8, ch: i32) -> *mut u8;

    #[nid(0x243665ED)]
    unsafe fn rindex(str: *const u8, ch: i32) -> *mut u8;

    #[nid(0x68A78817)]
    unsafe fn memchr(src: *const u8, byte: i32, len: SceSize) -> *mut u8;

    #[nid(0x81D0D1F7)]
    unsafe fn memcmp(src: *mut u8, other: *mut u8, len: SceSize) -> i32;

    #[nid(0xAB7592FF)]
    unsafe fn memcpy(dest: *mut u8, src: *const u8, len: SceSize) -> *mut u8;

    #[nid(0xA48D2592)]
    unsafe fn memmove(dest: *mut u8, src: *const u8, len: SceSize) -> *mut u8;

    #[nid(0x10F3BB61)]
    unsafe fn memset(src: *mut u8, val: i32, len: SceSize) -> *mut u8;

    #[nid(0xC2145E80)]
    unsafe fn snprintf(str: *mut u8, len: SceSize, format: *const u8, ...) -> i32;

    #[nid(0x7661E728)]
    unsafe fn sprintf(str: *mut u8, format: *const u8, ...) -> i32;

    #[nid(0x476FD94A)]
    unsafe fn strcat(dest: *mut u8, src: *const u8) -> *mut u8;

    #[nid(0xB1DC2AE8)]
    unsafe fn strchr(str: *const u8, ch: i32) -> *mut u8;

    #[nid(0xC0AB8932)]
    unsafe fn strcmp(str1: *const u8, str2: *const u8) -> i32;

    #[nid(0xEC6F1CF2)]
    unsafe fn strcpy(dest: *mut u8, src: *const u8) -> *mut u8;

    #[nid(0x52DF196C)]
    unsafe fn strlen(str: *const u8) -> SceSize;

    #[nid(0x7AB35214)]
    unsafe fn strncmp(str1: *const u8, str2: *const u8, max_size: SceSize) -> i32;

    #[nid(0xB49A7697)]
    unsafe fn strncpy(dest: *mut u8, src: *const u8, max_size: SceSize) -> *mut u8;

    #[nid(0x90C5573D)]
    unsafe fn strnlen(str: *const u8, max_size: SceSize) -> SceSize;

    #[nid(0x0DFB7B6C)]
    unsafe fn strpbrk(str: *const u8, break_set: *const u8) -> *mut u8;

    #[nid(0x4C0E0274)]
    unsafe fn strrchr(str: *const u8, ch: i32) -> *mut u8;

    #[nid(0x0D188658)]
    unsafe fn strstr(str: *const u8, pattern: *const u8) -> *mut u8;

    #[nid(0x47DD934D)]
    unsafe fn strtol(str: *const u8, end_ptr: *mut *mut u8, base: i32) -> i32;

    #[nid(0x6A7900E1)]
    unsafe fn strtoul(str: *const u8, end_ptr: *mut *mut u8, base: i32) -> u32;

    #[nid(0x3EC5BBF6)]
    fn tolower(ch: i32) -> i32;

    #[nid(0xCE2F7487)]
    fn toupper(ch: i32) -> i32;

    #[nid(0x1493EBD9)]
    unsafe fn wmemset(str: *mut u16, val: u16, len: SceSize) -> *mut u16;

    #[nid(0x7DEE14DE)]
    fn __udivdi3(a: u32, b: u32) -> u32;

    #[nid(0xDF17F4A2)]
    unsafe fn __udivmoddi4(a: u32, b: u32, c: *mut u32) -> u32;

    #[nid(0x5E8E5F42)]
    fn __umoddi3(a: u32, b: u32) -> u32;

    #[nid(0x1D83F344)]
    unsafe fn atob(a0: *mut u8, a1: *mut i32);

    #[nid(0xBC7554DF)]
    unsafe fn strcasecmp(str1: *const u8, str2: *const u8) -> i32;

    #[nid(0xAC7554DF)]
    unsafe fn strcasecmp(str1: *const u8, str2: *const u8, len: SceSize) -> i32;

    #[nid(0x983B00FB)]
    unsafe fn lowerString(orig: *const u8, ret: *mut u8, len: SceSize);

    #[nid(0x87F8D2DA)]
    unsafe fn strtok(str: *mut u8, seps: *const u8) -> *mut u8;

    #[nid(0x1AB53A58)]
    unsafe fn strtok_r(str: *mut u8, seps: *const u8, ctx: *mut *mut u8) -> *mut u8;

    #[nid(0xD3D1A3B9)]
    unsafe fn strncat(dest: *mut u8, src: *const u8, len: SceSize) -> *mut u8;

    #[nid(0xEFB593C9)]
    unsafe fn strncat_s(dest: *mut u8, num_elem: SceSize, src: *const u8, len: SceSize) -> *mut u8;

    #[nid(0x5ABF13F5)]
    unsafe fn strncpy_s(dest: *mut u8, num_elem: SceSize, src: *const u8, len: SceSize) -> SceSize;
}


// Libc functions that we need for user-space without using CFW's `SysclibForUser`

#[unsafe(no_mangle)]
#[cfg(all(feature = "non-stub-code", not(feature = "kernel")))]
#[cfg(not(feature = "cfw-api"))]
unsafe extern "C" fn memset(ptr: *mut u8, value: u32, num: SceSize) -> *mut u8 {
    unsafe {
        let mut i = 0;

        while i < num {
            *((ptr as SceSize + i) as *mut u8) = value as u8;
            i += 1;
        }

        ptr
    }
}

#[unsafe(no_mangle)]
#[cfg(all(feature = "non-stub-code", not(feature = "kernel")))]
#[cfg(not(feature = "cfw-api"))]
unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, num: SceSize) -> *mut u8 {
    unsafe {
        let mut i = 0;

        while i < num {
            *((dst as SceSize + i) as *mut u8) = *((src as SceSize + i) as *mut u8);
            i += 1;
        }

        dst
    }
}

#[unsafe(no_mangle)]
#[cfg(all(feature = "non-stub-code", not(feature = "kernel")))]
#[cfg(not(feature = "cfw-api"))]
unsafe extern "C" fn memcmp(ptr1: *mut u8, ptr2: *mut u8, num: SceSize) -> i32 {
    unsafe {
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

#[unsafe(no_mangle)]
#[cfg(all(feature = "non-stub-code", not(feature = "kernel")))]
#[cfg(not(feature = "cfw-api"))]
unsafe extern "C" fn memmove(dst: *mut u8, src: *mut u8, num: SceSize) -> *mut u8 {
    unsafe {
        if dst < src {
            let mut i = 0;

            while i < num {
                *((dst as SceSize + i) as *mut u8) = *((src as SceSize + i) as *mut u8);
                i += 1;
            }
        } else {
            let mut i = num - 1;

            while i >= 0 {
                *((dst as SceSize + i) as *mut u8) = *((src as SceSize + i) as *mut u8);
                i -= 1;
            }
        }

        dst
    }
}

#[unsafe(no_mangle)]
#[cfg(all(feature = "non-stub-code", not(feature = "kernel")))]
#[cfg(not(feature = "cfw-api"))]
unsafe extern "C" fn strlen(s: *mut u8) -> SceSize {
    unsafe {
        let mut len = 0;

        while *s.wrapping_add(len) != 0 {
            len += 1;
        }

        len as SceSize
    }
}

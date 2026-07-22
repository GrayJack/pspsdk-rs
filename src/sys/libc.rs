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
#[psp_stub(libname = "SysclibForKernel", flags = 0x0001, version = (0x00, 0x00), use_crate)]
unsafe extern "C" {
    #[nid(0x7F8A6F23)]
    pub unsafe fn bcmp(source: *const u8, other: *const u8, len: SceSize) -> i32;

    #[nid(0x097049BD)]
    pub unsafe fn bcopy(src: *const u8, dest: *mut u8, len: SceSize);

    #[nid(0x86FEFCE9)]
    pub unsafe fn bzero(str: *mut u8, len: SceSize);

    #[nid(0xD1CD40E5)]
    pub unsafe fn index(str: *const u8, ch: i32) -> *mut u8;

    #[nid(0x243665ED)]
    pub unsafe fn rindex(str: *const u8, ch: i32) -> *mut u8;

    #[nid(0x68A78817)]
    pub unsafe fn memchr(src: *const u8, byte: i32, len: SceSize) -> *mut u8;

    #[nid(0x81D0D1F7)]
    pub unsafe fn memcmp(src: *const u8, other: *const u8, len: SceSize) -> i32;

    #[nid(0xAB7592FF)]
    pub unsafe fn memcpy(dest: *mut u8, src: *const u8, len: SceSize) -> *mut u8;

    #[nid(0xA48D2592)]
    pub unsafe fn memmove(dest: *mut u8, src: *const u8, len: SceSize) -> *mut u8;

    #[nid(0x10F3BB61)]
    pub unsafe fn memset(src: *mut u8, val: i32, len: SceSize) -> *mut u8;

    #[nid(0xC2145E80)]
    pub unsafe fn snprintf(str: *mut u8, len: SceSize, format: *const u8, ...) -> i32;

    #[nid(0x7661E728)]
    pub unsafe fn sprintf(str: *mut u8, format: *const u8, ...) -> i32;

    #[nid(0x476FD94A)]
    pub unsafe fn strcat(dest: *mut u8, src: *const u8) -> *mut u8;

    #[nid(0xB1DC2AE8)]
    pub unsafe fn strchr(str: *const u8, ch: i32) -> *mut u8;

    #[nid(0xC0AB8932)]
    pub unsafe fn strcmp(str1: *const u8, str2: *const u8) -> i32;

    #[nid(0xEC6F1CF2)]
    pub unsafe fn strcpy(dest: *mut u8, src: *const u8) -> *mut u8;

    #[nid(0x52DF196C)]
    pub unsafe fn strlen(str: *const u8) -> SceSize;

    #[nid(0x7AB35214)]
    pub unsafe fn strncmp(str1: *const u8, str2: *const u8, max_size: SceSize) -> i32;

    #[nid(0xB49A7697)]
    pub unsafe fn strncpy(dest: *mut u8, src: *const u8, max_size: SceSize) -> *mut u8;

    #[nid(0x90C5573D)]
    pub unsafe fn strnlen(str: *const u8, max_size: SceSize) -> SceSize;

    #[nid(0x0DFB7B6C)]
    pub unsafe fn strpbrk(str: *const u8, break_set: *const u8) -> *mut u8;

    #[nid(0x4C0E0274)]
    pub unsafe fn strrchr(str: *const u8, ch: i32) -> *mut u8;

    #[nid(0x0D188658)]
    pub unsafe fn strstr(str: *const u8, pattern: *const u8) -> *mut u8;

    #[nid(0x47DD934D)]
    pub unsafe fn strtol(str: *const u8, end_ptr: *mut *mut u8, base: i32) -> i32;

    #[nid(0x6A7900E1)]
    pub unsafe fn strtoul(str: *const u8, end_ptr: *mut *mut u8, base: i32) -> u32;

    #[nid(0x3EC5BBF6)]
    pub fn tolower(ch: i32) -> i32;

    #[nid(0xCE2F7487)]
    pub fn toupper(ch: i32) -> i32;

    #[nid(0x1493EBD9)]
    pub unsafe fn wmemset(str: *mut u16, val: u16, len: SceSize) -> *mut u16;

    #[nid(0x7DEE14DE)]
    pub fn __udivdi3(a: u32, b: u32) -> u32;

    #[nid(0xDF17F4A2)]
    pub unsafe fn __udivmoddi4(a: u32, b: u32, c: *mut u32) -> u32;

    #[nid(0x5E8E5F42)]
    pub fn __umoddi3(a: u32, b: u32) -> u32;
}

#[cfg(all(not(feature = "kernel"), feature = "cfw-api"))]
#[psp_stub(libname = "SysclibForUser", flags = 0x4001, version = (0x00, 0x00), use_crate)]
unsafe extern "C" {
    #[nid(0x7F8A6F23)]
    pub unsafe fn bcmp(source: *const u8, other: *const u8, len: SceSize) -> i32;

    #[nid(0x097049BD)]
    pub unsafe fn bcopy(src: *const u8, dest: *mut u8, len: SceSize);

    #[nid(0x86FEFCE9)]
    pub unsafe fn bzero(str: *mut u8, len: SceSize);

    #[nid(0xD1CD40E5)]
    pub unsafe fn index(str: *const u8, ch: i32) -> *mut u8;

    #[nid(0x243665ED)]
    pub unsafe fn rindex(str: *const u8, ch: i32) -> *mut u8;

    #[nid(0x68A78817)]
    pub unsafe fn memchr(src: *const u8, byte: i32, len: SceSize) -> *mut u8;

    #[nid(0x81D0D1F7)]
    pub unsafe fn memcmp(src: *mut u8, other: *mut u8, len: SceSize) -> i32;

    #[nid(0xAB7592FF)]
    pub unsafe fn memcpy(dest: *mut u8, src: *const u8, len: SceSize) -> *mut u8;

    #[nid(0xA48D2592)]
    pub unsafe fn memmove(dest: *mut u8, src: *const u8, len: SceSize) -> *mut u8;

    #[nid(0x10F3BB61)]
    pub unsafe fn memset(src: *mut u8, val: i32, len: SceSize) -> *mut u8;

    #[nid(0xC2145E80)]
    pub unsafe fn snprintf(str: *mut u8, len: SceSize, format: *const u8, ...) -> i32;

    #[nid(0x7661E728)]
    pub unsafe fn sprintf(str: *mut u8, format: *const u8, ...) -> i32;

    #[nid(0x476FD94A)]
    pub unsafe fn strcat(dest: *mut u8, src: *const u8) -> *mut u8;

    #[nid(0xB1DC2AE8)]
    pub unsafe fn strchr(str: *const u8, ch: i32) -> *mut u8;

    #[nid(0xC0AB8932)]
    pub unsafe fn strcmp(str1: *const u8, str2: *const u8) -> i32;

    #[nid(0xEC6F1CF2)]
    pub unsafe fn strcpy(dest: *mut u8, src: *const u8) -> *mut u8;

    #[nid(0x52DF196C)]
    pub unsafe fn strlen(str: *const u8) -> SceSize;

    #[nid(0x7AB35214)]
    pub unsafe fn strncmp(str1: *const u8, str2: *const u8, max_size: SceSize) -> i32;

    #[nid(0xB49A7697)]
    pub unsafe fn strncpy(dest: *mut u8, src: *const u8, max_size: SceSize) -> *mut u8;

    #[nid(0x90C5573D)]
    pub unsafe fn strnlen(str: *const u8, max_size: SceSize) -> SceSize;

    #[nid(0x0DFB7B6C)]
    pub unsafe fn strpbrk(str: *const u8, break_set: *const u8) -> *mut u8;

    #[nid(0x4C0E0274)]
    pub unsafe fn strrchr(str: *const u8, ch: i32) -> *mut u8;

    #[nid(0x0D188658)]
    pub unsafe fn strstr(str: *const u8, pattern: *const u8) -> *mut u8;

    #[nid(0x47DD934D)]
    pub unsafe fn strtol(str: *const u8, end_ptr: *mut *mut u8, base: i32) -> i32;

    #[nid(0x6A7900E1)]
    pub unsafe fn strtoul(str: *const u8, end_ptr: *mut *mut u8, base: i32) -> u32;

    #[nid(0x3EC5BBF6)]
    pub fn tolower(ch: i32) -> i32;

    #[nid(0xCE2F7487)]
    pub fn toupper(ch: i32) -> i32;

    #[nid(0x1493EBD9)]
    pub unsafe fn wmemset(str: *mut u16, val: u16, len: SceSize) -> *mut u16;

    #[nid(0x7DEE14DE)]
    pub fn __udivdi3(a: u32, b: u32) -> u32;

    #[nid(0xDF17F4A2)]
    pub unsafe fn __udivmoddi4(a: u32, b: u32, c: *mut u32) -> u32;

    #[nid(0x5E8E5F42)]
    pub fn __umoddi3(a: u32, b: u32) -> u32;

    #[nid(0x1D83F344)]
    pub unsafe fn atob(a0: *mut u8, a1: *mut i32);

    #[nid(0xBC7554DF)]
    pub unsafe fn strcasecmp(str1: *const u8, str2: *const u8) -> i32;

    #[nid(0xAC7554DF)]
    pub unsafe fn strncasecmp(str1: *const u8, str2: *const u8, len: SceSize) -> i32;

    #[nid(0x983B00FB)]
    pub unsafe fn lowerString(orig: *const u8, ret: *mut u8, len: SceSize);

    #[nid(0x87F8D2DA)]
    pub unsafe fn strtok(str: *mut u8, seps: *const u8) -> *mut u8;

    #[nid(0x1AB53A58)]
    pub unsafe fn strtok_r(str: *mut u8, seps: *const u8, ctx: *mut *mut u8) -> *mut u8;

    #[nid(0xD3D1A3B9)]
    pub unsafe fn strncat(dest: *mut u8, src: *const u8, len: SceSize) -> *mut u8;

    #[nid(0xEFB593C9)]
    pub unsafe fn strncat_s(
        dest: *mut u8, num_elem: SceSize, src: *const u8, len: SceSize,
    ) -> *mut u8;

    #[nid(0x5ABF13F5)]
    pub unsafe fn strncpy_s(
        dest: *mut u8, num_elem: SceSize, src: *const u8, len: SceSize,
    ) -> SceSize;
}

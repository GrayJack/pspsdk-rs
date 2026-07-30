//! This optimizations knowledge came from the compiler-builtins crate
#![expect(
    unsafe_op_in_unsafe_fn,
    reason = "Some implementations are dependent on pointer operations"
)]
#![allow(unused, reason = "feature flags dependable")]

use core::ffi::c_char;

const WORD_SIZE: usize = core::mem::size_of::<usize>();
const WORD_MASK: usize = WORD_SIZE - 1;

// If the number of bytes involved exceed this threshold we will opt in word-wise copy.
// The value here selected is max(2 * WORD_SIZE, 16):
// * We need at least 2 * WORD_SIZE bytes to guarantee that at least 1 word will be copied through
//   word-wise copy.
// * The word-wise copy logic needs to perform some checks so it has some small overhead. ensures
//   that even on 32-bit platforms we have copied at least 8 bytes through word-wise copy so the
//   saving of word-wise copy outweights the fixed overhead.
// #[cfg(not(target_arch = "x86_64"))]
const WORD_COPY_THRESHOLD: usize = if 2 * WORD_SIZE > 16 { 2 * WORD_SIZE } else { 16 };


/// Loads a `T`-sized chunk from `src` into `dst` at offset `offset`, if that does not exceed
/// `load_sz`. The offset pointers must both be `T`-aligned. Returns the new offset, advanced by the
/// chunk size if a load happened.
#[inline(always)]
#[cfg(not(feature = "opt-size-c"))]
unsafe fn load_chunk_aligned<T: Copy>(
    src: *const usize, dst: *mut usize, load_sz: usize, offset: usize,
) -> usize {
    let chunk_sz = core::mem::size_of::<T>();
    if (load_sz & chunk_sz) != 0 {
        *dst.wrapping_byte_add(offset).cast::<T>() = *src.wrapping_byte_add(offset).cast::<T>();
        offset | chunk_sz
    } else {
        offset
    }
}

/// Load `load_sz` many bytes from `src`, which must be usize-aligned. Acts as if we did a `usize`
/// read with the out-of-bounds part filled with 0s.
/// `load_sz` be strictly less than `WORD_SIZE`.
#[inline(always)]
#[cfg(not(feature = "opt-size-c"))]
unsafe fn load_aligned_partial(src: *const usize, load_sz: usize) -> usize {
    debug_assert!(load_sz < WORD_SIZE);
    // We can read up to 7 bytes here, which is enough for WORD_SIZE of 8
    // (since `load_sz < WORD_SIZE`).
    const { assert!(WORD_SIZE <= 8) };

    let mut i = 0;
    let mut out = 0usize;
    // We load in decreasing order, so the pointers remain sufficiently aligned for the next step.
    i = load_chunk_aligned::<u32>(src, &raw mut out, load_sz, i);
    i = load_chunk_aligned::<u16>(src, &raw mut out, load_sz, i);
    i = load_chunk_aligned::<u8>(src, &raw mut out, load_sz, i);
    debug_assert!(i == load_sz);
    out
}

/// Load `load_sz` many bytes from `src.wrapping_byte_add(WORD_SIZE - load_sz)`. `src` must be
/// `usize`-aligned. The bytes are returned as the *last* bytes of the return value, i.e., this acts
/// as if we had done a `usize` read from `src`, with the out-of-bounds part filled with 0s.
/// `load_sz` be strictly less than `WORD_SIZE`.
#[inline(always)]
#[cfg(not(feature = "opt-size-c"))]
unsafe fn load_aligned_end_partial(src: *const usize, load_sz: usize) -> usize {
    debug_assert!(load_sz < WORD_SIZE);
    // We can read up to 7 bytes here, which is enough for WORD_SIZE of 8
    // (since `load_sz < WORD_SIZE`).
    const { assert!(WORD_SIZE <= 8) };

    let mut i = 0;
    let mut out = 0usize;
    // Obtain pointers pointing to the beginning of the range we want to load.
    let src_shifted = src.wrapping_byte_add(WORD_SIZE - load_sz);
    let out_shifted = (&raw mut out).wrapping_byte_add(WORD_SIZE - load_sz);
    // We load in increasing order, so by the time we reach `u16` things are 2-aligned etc.
    i = load_chunk_aligned::<u8>(src_shifted, out_shifted, load_sz, i);
    i = load_chunk_aligned::<u16>(src_shifted, out_shifted, load_sz, i);
    i = load_chunk_aligned::<u32>(src_shifted, out_shifted, load_sz, i);
    debug_assert!(i == load_sz);
    out
}

#[inline(always)]
#[cfg(feature = "opt-size-c")]
pub unsafe fn compare_bytes(mut src: *const u8, mut other: *const u8, mut len: usize) -> i32 {
    let mut n = 0;
    while n < len {
        let a = unsafe { *src.wrapping_add(n) };
        let b = unsafe { *other.wrapping_add(n) };
        if a != b {
            return a as i32 - b as i32;
        }
        n += 1;
    }
    0
}

#[inline(always)]
#[cfg(not(feature = "opt-size-c"))]
pub unsafe fn compare_bytes(mut src: *const u8, mut other: *const u8, mut len: usize) -> i32 {
    #[inline(always)]
    fn cmp_bytes(src: *const u8, other: *const u8, len: usize) -> i32 {
        let mut n = 0;
        while n < len {
            let a = unsafe { *src.wrapping_add(n) };
            let b = unsafe { *other.wrapping_add(n) };
            if a != b {
                return a as i32 - b as i32;
            }
            n += 1;
        }
        0
    }

    #[inline(always)]
    fn cmp_words(src: *const u8, other: *const u8, mut len: usize) -> i32 {
        let mut wsrc = src as *const usize;
        let mut wother = other as *const usize;
        while len >= WORD_SIZE {
            if unsafe { *wsrc != *wother } {
                // If words are not equal, compare byte by byte
                let src_bytes = wsrc as *const u8;
                let other_bytes = wother as *const u8;
                for i in 0..WORD_SIZE {
                    let a = unsafe { *src_bytes.wrapping_add(i) };
                    let b = unsafe { *other_bytes.wrapping_add(i) };
                    if a != b {
                        return a as i32 - b as i32;
                    }
                }
            }
            wsrc = wsrc.wrapping_add(1);
            wother = wother.wrapping_add(1);
            len -= WORD_SIZE;
        }
        0
    }

    // this branch is always removed when debug assertions are not enabled.
    if len > 0 {
        debug_assert!(!src.is_null());
        debug_assert!(!other.is_null());
    }

    if len >= WORD_COPY_THRESHOLD {
        // Align `src` and `dest`
        while len > 0 && (src as usize | other as usize) & WORD_MASK != 0 {
            if *src != *other {
                return (*src as i32) - (*other as i32);
            }
            src = src.wrapping_add(1);
            other = other.wrapping_add(1);
            len -= 1;
        }

        // compare words
        let word_cmp_result = cmp_words(src, other, len);
        if word_cmp_result != 0 {
            return word_cmp_result;
        }

        // Update pointers and size after word comparison
        let word_size = len & !WORD_MASK;
        src = src.wrapping_add(word_size);
        other = other.wrapping_add(word_size);
        len -= word_size;
    }

    cmp_bytes(src, other, len)
}

#[inline(always)]
#[cfg(feature = "opt-size-c")]
pub unsafe fn copy_forward(mut dest: *mut u8, mut src: *const u8, mut len: usize) {
    let dest_end = dest.wrapping_add(len);
    while dest < dest_end {
        *dest = *src;
        dest = dest.wrapping_add(1);
        src = src.wrapping_add(1);
    }
}

#[inline(always)]
#[cfg(not(feature = "opt-size-c"))]
pub unsafe fn copy_forward(mut dest: *mut u8, mut src: *const u8, mut len: usize) {
    #[inline(always)]
    unsafe fn copy_forward_bytes(mut dest: *mut u8, mut src: *const u8, len: usize) {
        let dest_end = dest.wrapping_add(len);
        while dest < dest_end {
            *dest = *src;
            dest = dest.wrapping_add(1);
            src = src.wrapping_add(1);
        }
    }

    #[inline(always)]
    unsafe fn copy_forward_aligned_words(dest: *mut u8, src: *const u8, len: usize) {
        let mut w_dest = dest as *mut usize;
        let mut w_src = src as *mut usize;
        let dest_end = dest.wrapping_add(len) as *mut usize;

        while w_dest < dest_end {
            *w_dest = *w_src;
            w_dest = w_dest.wrapping_add(1);
            w_src = w_src.wrapping_add(1);
        }
    }

    /// `n` is in units of bytes, but must be a multiple of the word size and must not be 0.
    /// `src` *must not* be `usize`-aligned.
    #[inline(always)]
    unsafe fn copy_forward_misaligned_words(dest: *mut u8, src: *const u8, len: usize) {
        debug_assert!(len > 0 && len.is_multiple_of(WORD_SIZE));
        debug_assert!(!src.addr().is_multiple_of(WORD_SIZE));

        let mut w_dest = dest as *mut usize;
        let dest_end = dest.wrapping_add(len) as *mut usize;

        // Calculate the misalignment offset and shift needed to reassemble value.
        // Since `src` is definitely not aligned, `offset` is in the range 1..WORD_SIZE.
        let offset = src as usize & WORD_MASK;
        let shift = offset * 8;

        // Realign src
        let mut src_aligned = (src as usize & !WORD_MASK) as *mut usize;
        let mut prev_word = load_aligned_end_partial(src_aligned, WORD_SIZE - offset);

        while w_dest.wrapping_add(1) < dest_end {
            src_aligned = src_aligned.wrapping_add(1);
            let cur_word = *src_aligned;
            let reassembled = if cfg!(target_endian = "little") {
                prev_word >> shift | cur_word << (WORD_SIZE * 8 - shift)
            } else {
                prev_word << shift | cur_word >> (WORD_SIZE * 8 - shift)
            };
            prev_word = cur_word;

            *w_dest = reassembled;
            w_dest = w_dest.wrapping_add(1);
        }

        // There's one more element left to go, and we can't use the loop for that as on the `src`
        // side, it is partially out-of-bounds.
        src_aligned = src_aligned.wrapping_add(1);
        let cur_word = load_aligned_partial(src_aligned, offset);
        let reassembled = if cfg!(target_endian = "little") {
            prev_word >> shift | cur_word << (WORD_SIZE * 8 - shift)
        } else {
            prev_word << shift | cur_word >> (WORD_SIZE * 8 - shift)
        };
        // prev_word does not matter any more

        *w_dest = reassembled;
        // dest_usize does not matter any more
    }

    // this branch is always removed when debug assertions are not enabled.
    if len > 0 {
        debug_assert!(!src.is_null());
        debug_assert!(!dest.is_null());
    }

    if len >= WORD_COPY_THRESHOLD {
        // Align `dest`
        // Because of len >= 2 * WORD_SIZE, dst_misalignment < len
        let dest_misaligment_size = (dest as usize).wrapping_neg() & WORD_MASK;
        copy_forward_bytes(dest, src, dest_misaligment_size);
        dest = dest.wrapping_add(dest_misaligment_size);
        src = src.wrapping_add(dest_misaligment_size);
        len -= dest_misaligment_size;

        let len_words = len & !WORD_MASK;
        let src_misalignment_size = src as usize & WORD_MASK;
        if src_misalignment_size == 0 {
            copy_forward_aligned_words(dest, src, len_words);
        } else {
            copy_forward_misaligned_words(dest, src, len_words);
        }
        dest = dest.wrapping_add(len_words);
        src = src.wrapping_add(len_words);
        len -= len_words;
    }

    copy_forward_bytes(dest, src, len)
}

#[inline(always)]
#[cfg(feature = "opt-size-c")]
pub unsafe fn copy_backward(mut dest: *mut u8, mut src: *const u8, mut len: usize) {
    let dest_start = dest.wrapping_sub(len);
    while dest_start < dest {
        dest = dest.wrapping_sub(1);
        src = src.wrapping_sub(1);
        *dest = *src;
    }
}

#[inline(always)]
#[cfg(not(feature = "opt-size-c"))]
pub unsafe fn copy_backward(dest: *mut u8, src: *const u8, mut len: usize) {
    #[inline(always)]
    unsafe fn copy_backward_bytes(mut dest: *mut u8, mut src: *const u8, len: usize) {
        let dest_start = dest.wrapping_sub(len);
        while dest_start < dest {
            dest = dest.wrapping_sub(1);
            src = src.wrapping_sub(1);
            *dest = *src;
        }
    }

    #[inline(always)]
    unsafe fn copy_backward_aligned_words(dest: *mut u8, src: *const u8, len: usize) {
        let mut w_dest = dest as *mut usize;
        let mut w_src = src as *mut usize;
        let dest_start = dest.wrapping_sub(len) as *mut usize;

        while dest_start < w_dest {
            w_dest = w_dest.wrapping_sub(1);
            w_src = w_src.wrapping_sub(1);
            *w_dest = *w_src;
        }
    }

    /// `n` is in units of bytes, but must be a multiple of the word size and must not be 0.
    /// `src` *must not* be `usize`-aligned.
    #[inline(always)]
    unsafe fn copy_backward_misaligned_words(dest: *mut u8, src: *const u8, n: usize) {
        debug_assert!(n > 0 && n.is_multiple_of(WORD_SIZE));
        debug_assert!(!src.addr().is_multiple_of(WORD_SIZE));

        let mut w_dest = dest as *mut usize;
        let dest_start = dest.wrapping_sub(n) as *mut usize;

        // Calculate the misalignment offset and shift needed to reassemble value.
        // Since `src` is definitely not aligned, `offset` is in the range 1..WORD_SIZE.
        let offset = src as usize & WORD_MASK;
        let shift = offset * 8;

        // Realign src_aligned
        let mut src_aligned = src.wrapping_byte_sub(offset) as *mut usize;
        let mut prev_word = load_aligned_partial(src_aligned, offset);

        while dest_start.wrapping_add(1) < w_dest {
            src_aligned = src_aligned.wrapping_sub(1);
            let cur_word = *src_aligned;
            let reassembled = if cfg!(target_endian = "little") {
                prev_word << (WORD_SIZE * 8 - shift) | cur_word >> shift
            } else {
                prev_word >> (WORD_SIZE * 8 - shift) | cur_word << shift
            };
            prev_word = cur_word;

            w_dest = w_dest.wrapping_sub(1);
            *w_dest = reassembled;
        }

        // There's one more element left to go, and we can't use the loop for that as on the `src`
        // side, it is partially out-of-bounds.
        src_aligned = src_aligned.wrapping_sub(1);
        let cur_word = load_aligned_end_partial(src_aligned, WORD_SIZE - offset);
        let reassembled = if cfg!(target_endian = "little") {
            prev_word << (WORD_SIZE * 8 - shift) | cur_word >> shift
        } else {
            prev_word >> (WORD_SIZE * 8 - shift) | cur_word << shift
        };
        // prev_word does not matter any more

        w_dest = w_dest.wrapping_sub(1);
        *w_dest = reassembled;
    }

    // this branch is always removed when debug assertions are not enabled.
    if len > 0 {
        debug_assert!(!src.is_null());
        debug_assert!(!dest.is_null());
    }

    let mut dest = dest.wrapping_add(len);
    let mut src = src.wrapping_add(len);

    if len >= WORD_COPY_THRESHOLD {
        // Align `dest`
        // Because of len >= 2 * WORD_SIZE, dst_misalignment < len
        let dest_misalignment_size = dest as usize & WORD_MASK;
        copy_backward_bytes(dest, src, dest_misalignment_size);
        dest = dest.wrapping_sub(dest_misalignment_size);
        src = src.wrapping_sub(dest_misalignment_size);
        len -= dest_misalignment_size;

        let len_words = len & !WORD_MASK;
        let src_misalignment_size = src as usize & WORD_MASK;
        if src_misalignment_size == 0 {
            copy_backward_aligned_words(dest, src, len_words);
        } else {
            copy_backward_misaligned_words(dest, src, len_words);
        }
        dest = dest.wrapping_sub(len_words);
        src = src.wrapping_sub(len_words);
        len -= len_words;
    }

    copy_backward_bytes(dest, src, len);
}

#[inline(always)]
#[cfg(feature = "opt-size-c")]
pub unsafe fn set_bytes(mut dest: *mut u8, byte: u8, mut len: usize) {
    let end = dest.wrapping_add(len);
    while dest < end {
        *dest = byte;
        dest = dest.wrapping_add(1);
    }
}

#[inline(always)]
#[cfg(not(feature = "opt-size-c"))]
pub unsafe fn set_bytes(mut dest: *mut u8, byte: u8, mut len: usize) {
    #[inline(always)]
    unsafe fn set_bytes_bytes(mut dest: *mut u8, byte: u8, len: usize) {
        let end = dest.wrapping_add(len);
        while dest < end {
            *dest = byte;
            dest = dest.wrapping_add(1);
        }
    }

    #[inline(always)]
    unsafe fn set_bytes_words(dest: *mut u8, byte: u8, len: usize) {
        let mut broadcast = byte as usize;
        let mut bits = 8;
        while bits < WORD_SIZE * 8 {
            broadcast |= broadcast << bits;
            bits *= 2;
        }

        let mut s_usize = dest as *mut usize;
        let end = dest.wrapping_add(len) as *mut usize;

        while s_usize < end {
            *s_usize = broadcast;
            s_usize = s_usize.wrapping_add(1);
        }
    }

    // this branch is always removed when debug assertions are not enabled.
    if len > 0 {
        debug_assert!(!dest.is_null());
    }

    if len >= WORD_COPY_THRESHOLD {
        // Align `dest`
        // Because of len >= 2 * WORD_SIZE, dst_misalignment < len
        let misaligment_size = (dest as usize).wrapping_neg() & WORD_MASK;
        set_bytes_bytes(dest, byte, misaligment_size);
        dest = dest.wrapping_add(misaligment_size);
        len -= misaligment_size;

        let len_words = len & !WORD_MASK;
        set_bytes_words(dest, byte, len_words);
        dest = dest.wrapping_add(len_words);
        len -= len_words;
    }

    set_bytes_bytes(dest, byte, len)
}

// String operations

const MASK01: usize = 0x01010101;
const MASK80: usize = 0x80808080;
const USIZE_MASK: usize = size_of::<usize>() - 1;

#[inline(always)]
// #[cfg(not(feature = "opt-size-c"))] // This one a naive impl is actually causes binary to be
// bigger on PSP
pub unsafe fn c_string_length(str: *const c_char) -> usize {
    let mut p = str;

    // Skip the first few bytes until we have an aligned `p`
    while (p.addr() & USIZE_MASK) != 0 {
        if *p == 0 {
            return p.offset_from(str) as usize;
        }
        p = p.wrapping_add(1);
    }

    // Scan the rest of the string using word sized operation
    let mut lp: *const usize = p.cast();
    loop {
        let word = lp.read_unaligned();

        let has_null = (word.wrapping_sub(MASK01) & MASK80) != 0;
        if has_null {
            p = lp.cast();

            for i in 0..size_of::<usize>() {
                let ptr = p.wrapping_add(i);
                if *ptr == 0 {
                    return ptr.offset_from(str) as usize;
                }
            }
        }

        lp = lp.wrapping_add(1);
    }
}

pub trait PointersOverlap: super::Sealed {
    /// Check if `self` and `other` pointer overleap in memory at `self + len * size_of::<T>()`
    /// pointer
    fn overlaps_with(self, other: Self, len: usize) -> bool;
}

impl<T> PointersOverlap for *const T {
    /// Check if `self` and `other` pointer overleap in memory at `self + len * size_of::<T>()`
    /// pointer
    #[inline(always)]
    fn overlaps_with(self, other: Self, len: usize) -> bool {
        // either `other` is far enough ahead of `self`, or `self` is ahead of `other` (and delta
        // overflowed).
        let delta = (other as usize).wrapping_sub(self as usize);
        delta < len
    }
}

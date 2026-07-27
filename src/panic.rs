//! Panic support for the PSP.

// Most of the code here is lifted from `rustc/src/libstd/panicking.rs`. It has
// been adapted to run on the PSP.
#![allow(unexpected_cfgs)]

#[cfg(feature = "std")]
use core::any::Any;
#[cfg(not(feature = "std"))]
use core::{
    any::Any,
    mem::{self, ManuallyDrop},
    panic::{self, Location, PanicInfo},
};

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String};
#[cfg(not(feature = "std"))]
use io_core::io::Write;

// Prints to the "panic output", depending on the platform this may be:
// - the standard error output
// - some dedicated platform specific output
// - nothing (so this macro is a no-op)
macro_rules! rtprintpanic {
    ($($t:tt)*) => {
        #[cfg(not(panic = "immediate-abort"))] {
            print(format_args!($($t)*));
        }

        #[cfg(panic = "immediate-abort")]
        {
            let _ = format_args!($($t)*);
        }
    }
}

macro_rules! rtabort {
    ($($t:tt)*) => {
        {
            print_and_die(format_args!("fatal runtime error: {}, aborting\n", format_args!($($t)*)));
        }
    }
}

#[cfg(not(feature = "std"))]
fn print(args: core::fmt::Arguments) {
    cfg_select! {
        not(panic = "immediate-abort") => {
            if cfg!(pbp) {
                crate::io::printing::_dprint(args);
            }

            if let Some(mut out) = crate::os::stdio::panic_output() {
                let _ = out.write_fmt(args);
            }
        },
        _ => {}
    }
}

#[cfg(not(feature = "std"))]
fn print_and_die(args: core::fmt::Arguments) -> ! {
    print(args);
    crate::process::abort();
}

#[inline(never)]
#[cfg(all(not(feature = "std"), panic = "immediate-abort"))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::process::abort()
}

#[inline(never)]
#[cfg(all(not(feature = "std"), panic = "abort"))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let message = info.message();
    let location = info.location().unwrap();

    update_panic_count(1);

    let panic_out = crate::os::stdio::panic_output();
    if let Some(mut panic_out) = panic_out {
        // Try to write the panic message to a buffer first to prevent other concurrent outputs
        // interleaving with it.
        let mut buffer = [0u8; 512];
        let mut cursor = crate::io::Cursor::new(&mut buffer[..]);

        let write_msg = |dst: &mut dyn Write| {
            // We add a newline to ensure the panic message appears at the start of a line.
            writeln!(dst, "\nPSPSDK: panicked at {location}:\n\t{message}")
        };

        if write_msg(&mut cursor).is_ok() {
            let pos = cursor.position() as usize;
            let _ = panic_out.write_all(&buffer[0..pos]);
        } else {
            // The message did not fit into the buffer, write it directly instead.
            let _ = write_msg(&mut panic_out);
        }
    }

    crate::process::abort()
}

#[inline(never)]
#[cfg(all(not(feature = "std"), panic = "unwind"))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic_impl(info)
}

#[inline(always)]
#[cfg_attr(not(target_os = "psp"), allow(unused))]
#[cfg(not(feature = "std"))]
fn panic_impl(info: &PanicInfo) -> ! {
    use core::fmt;

    struct FormatStringPayload<'a> {
        inner: &'a core::panic::PanicMessage<'a>,
        string: Option<String>,
    }

    impl FormatStringPayload<'_> {
        fn fill(&mut self) -> &mut String {
            let inner = self.inner;
            // Lazily, the first time this gets called, run the actual string formatting.
            self.string.get_or_insert_with(|| alloc::format!("{inner}"))
        }
    }

    unsafe impl panic::PanicPayload for FormatStringPayload<'_> {
        fn take_box(&mut self) -> *mut (dyn Any + Send) {
            // We do two allocations here, unfortunately. But (a) they're required with the current
            // scheme, and (b) we don't handle panic + OOM properly anyway (see comment in
            // begin_panic below).
            let contents = mem::take(self.fill());
            Box::into_raw(Box::new(contents))
        }

        fn get(&mut self) -> &(dyn Any + Send) {
            self.fill()
        }
    }

    impl fmt::Display for FormatStringPayload<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if let Some(s) = &self.string {
                f.write_str(s)
            } else {
                fmt::Display::fmt(&self.inner, f)
            }
        }
    }

    struct StaticStrPayload(&'static str);

    unsafe impl panic::PanicPayload for StaticStrPayload {
        fn take_box(&mut self) -> *mut (dyn Any + Send) {
            Box::into_raw(Box::new(self.0))
        }

        fn get(&mut self) -> &(dyn Any + Send) {
            &self.0
        }

        fn as_str(&mut self) -> Option<&str> {
            Some(self.0)
        }
    }

    impl fmt::Display for StaticStrPayload {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    let loc = info.location().unwrap(); // The current implementation always returns Some
    let msg = info.message();

    if let Some(s) = msg.as_str() {
        rust_panic_with_hook(&mut StaticStrPayload(s), loc);
    } else {
        rust_panic_with_hook(
            &mut FormatStringPayload {
                inner: &msg,
                string: None,
            },
            loc,
        );
    }
}

/// Central point for dispatching panics.
///
/// Executes the primary logic for a panic, including checking for recursive
/// panics, panic hooks, and finally dispatching to the panic runtime to either
/// abort or unwind.
#[cfg(not(feature = "std"))]
fn rust_panic_with_hook(
    payload: &mut dyn panic::PanicPayload, location: &'static Location<'static>,
) -> ! {
    let panics = update_panic_count(1);

    // payload.get(); // populate the payload's string

    let message: &str = payload.as_str().unwrap_or_default();
    rtprintpanic!(
        "panicked at {location}:\n{message}\nthread panicked while processing panic. aborting.\n"
    );

    if panics > 1 {
        // If a thread panics while it's already unwinding then we
        // have limited options. Currently our preference is to
        // just abort. In the future we may consider resuming
        // unwinding or otherwise exiting the thread cleanly.
        rtabort!("thread panicked while processing panic. aborting.");
    }

    rust_panic(payload)
}

/// Determines whether the current thread is unwinding because of panic.
#[inline]
pub fn panicking() -> bool {
    update_panic_count(0) > 0
}

fn update_panic_count(amt: isize) -> usize {
    // TODO: Make this thread local
    static mut PANIC_COUNT: usize = 0;

    unsafe {
        PANIC_COUNT = (PANIC_COUNT as isize + amt) as usize;
        PANIC_COUNT
    }
}

#[allow(improper_ctypes)]
unsafe extern "C" {
    #[cfg_attr(not(bootstrap), rustc_std_internal_symbol)]
    fn __rust_panic_cleanup(payload: *mut u8) -> *mut (dyn Any + Send + 'static);
}

#[allow(unexpected_cfgs)]
unsafe extern "Rust" {
    #[cfg_attr(not(bootstrap), rustc_std_internal_symbol)]
    fn __rust_start_panic(payload: &mut dyn panic::PanicPayload) -> u32;
}

#[cfg_attr(not(bootstrap), rustc_std_internal_symbol)]
extern "C" fn __rust_foreign_exception() -> ! {
    loop {
        core::hint::spin_loop()
    }
}

#[inline(never)]
#[unsafe(no_mangle)]
#[cfg(not(feature = "std"))]
fn rust_panic(msg: &mut dyn panic::PanicPayload) -> ! {
    let code = unsafe { __rust_start_panic(msg) };
    rtabort!("failed to initiate panic, error {}", code)
}

#[allow(unexpected_cfgs)]
#[cfg(not(feature = "std"))]
#[cfg_attr(not(bootstrap), rustc_std_internal_symbol)]
extern "C" fn __rust_drop_panic() -> ! {
    rtabort!("Rust panics must be rethrown");
}

#[cfg(all(feature = "std", not(target_os = "psp")))]
pub use std::panic::catch_unwind;

/// Invoke a closure, capturing the cause of an unwinding panic if one occurs.
#[inline(never)]
#[cfg(all(not(feature = "std"), panic = "immediate-abort"))]
pub fn catch_unwind<R, F: FnOnce() -> R>(f: F) -> Result<R, Box<dyn Any + Send>> {
    Ok(f())
}

/// Invoke a closure, capturing the cause of an unwinding panic if one occurs.
#[inline(never)]
#[cfg(all(not(feature = "std"), not(panic = "immediate-abort")))]
pub fn catch_unwind<R, F: FnOnce() -> R>(f: F) -> Result<R, Box<dyn Any + Send>> {
    // This whole function is directly lifted out of rustc. See comments there
    // for an explanation of how this actually works.

    union Data<F, R> {
        f: ManuallyDrop<F>,
        r: ManuallyDrop<R>,
        p: ManuallyDrop<Box<dyn Any + Send>>,
    }

    let mut data = Data {
        f: ManuallyDrop::new(f),
    };

    unsafe {
        return if core::intrinsics::catch_unwind(do_call, &raw mut data, do_catch) {
            Err(ManuallyDrop::into_inner(data.p))
        } else {
            Ok(ManuallyDrop::into_inner(data.r))
        };
    }

    #[cold]
    unsafe fn cleanup(payload: *mut u8) -> Box<dyn Any + Send + 'static> {
        let obj = unsafe { Box::from_raw(__rust_panic_cleanup(payload)) };
        update_panic_count(-1);
        obj
    }

    #[inline]
    unsafe fn do_call<F: FnOnce() -> R, R>(data: *mut Data<F, R>) {
        unsafe {
            let f = ManuallyDrop::take(&mut (*data).f);
            (*data).r = ManuallyDrop::new(f());
        }
    }

    #[inline]
    unsafe fn do_catch<F: FnOnce() -> R, R>(data: *mut Data<F, R>, payload: *mut u8) {
        unsafe {
            let obj = cleanup(payload);
            (*data).p = ManuallyDrop::new(obj);
        }
    }
}

// TODO: EH personality was moved from the panic_unwind crate to std in
// https://github.com/rust-lang/rust/pull/92845. This no-op implementation
// should be replaced with the version from std when using no_std.
#[cfg(not(feature = "std"))]
#[lang = "eh_personality"]
unsafe extern "C" fn rust_eh_personality() {}

#[cfg_attr(panic = "unwind", link(name = "unwind", kind = "static"))]
unsafe extern "C" {}

/// These symbols and functions should not actually be used. `libunwind`,
/// however, requires them to be present so that it can link.
// TODO: Patch these out of libunwind instead.
#[cfg(all(target_os = "psp", feature = "non-stub-code"))]
mod libunwind_shims {
    #[unsafe(no_mangle)]
    unsafe extern "C" fn fprintf(_stream: *const u8, _format: *const u8, _: ...) -> isize {
        -1
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn fflush(_stream: *const u8) -> i32 {
        -1
    }

    #[unsafe(no_mangle)]
    #[allow(deprecated)]
    unsafe extern "C" fn abort() {
        // loop {
        //     crate::sys::spin_loop();
        // }
        crate::process::abort()
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn malloc(size: usize) -> *mut u8 {
        use alloc::alloc::{alloc, Layout};

        let size = size + 4;

        unsafe {
            let data = alloc(Layout::from_size_align_unchecked(size, 4));
            *(data as *mut usize) = size;

            data.offset(4)
        }
    }


    #[unsafe(no_mangle)]
    unsafe extern "C" fn free(data: *mut u8) {
        use alloc::alloc::{dealloc, Layout};

        unsafe {
            let base = data.sub(4);
            let size = *(base as *mut usize);

            dealloc(base, Layout::from_size_align_unchecked(size, 4));
        }
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn getenv(_name: *const u8) -> *const u8 {
        core::ptr::null()
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn __assert_func(_: *const u8, _: i32, _: *const u8, _: *const u8) {}

    #[unsafe(no_mangle)]
    static _impure_ptr: [usize; 0] = [];
}

pub(crate) struct AssertUnwindSafe<T>(pub T);
impl<T> core::panic::UnwindSafe for AssertUnwindSafe<T> {}

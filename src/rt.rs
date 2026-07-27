//! Runtime services
//!
//! The `rt` module provides a narrow set of runtime services,
//! including the global heap (exported in `heap`) and unwinding and
//! backtrace support. The APIs in this module are highly unstable,
//! and should be considered as private implementation details for the
//! time being.

use core::cell::SyncUnsafeCell;

use crate::{
    panicking,
    sys::{self, thread::ThreadId},
};

const MAX_ARGC: usize = 19;

/// Process `argc_bytes` and `argp` from `module_start` and creates a `argc` and `argv` to pass to
/// be passed to be used later. when starting the module main thread.
///
/// # Safety
/// `argp` must be valid for `argc_bytes`. And this function is only to be used with `module_start`.
pub unsafe fn process_argc_argv(
    argc_bytes: usize, argp: *mut u8,
) -> (usize, [*mut u8; MAX_ARGC + 1]) {
    let mut argv: [*mut u8; 20] = [core::ptr::null_mut(); 20];
    let mut argc = 0;
    let mut loc = 0;
    let ptr: *mut u8 = argp.cast();

    while loc < argc_bytes {
        unsafe {
            argv[argc] = ptr.add(loc);

            let arg_len = crate::private::strlen(argv[argc].cast()) + 1;

            loc += arg_len;
            argc += 1;

            if argc == 19 {
                break;
            }
        }
    }

    (argc, argv)
}

/// Initialization of stuff required to live on `module_start`. Returns `argc` and `argv` tuple.
///
/// For barebones projects, it is the user responsibility to call this function on `module_start`.
///
/// # Safety
/// `argp` must be valid for `argc_bytes`. And this function is only to be used with `module_start`.
pub unsafe fn module_start_init(
    argc_bytes: usize, argp: *mut core::ffi::c_void,
) -> (usize, [*mut u8; MAX_ARGC + 1]) {
    // Set OS functions for pspsdk::io module
    crate::set_psp_os_functions();

    unsafe { process_argc_argv(argc_bytes, argp.cast()) }
}

/// Initialize the current working directory.
///
/// # Safety
/// `arg0` must be valid for null-terminated string.
pub unsafe fn init_cwd(arg0: *mut u8) {
    if arg0.is_null() {
        return;
    }

    unsafe {
        let mut len = 0;
        while *arg0.add(len) != 0 {
            len += 1;
        }

        // Truncate until last '/'
        while len > 0 && *arg0.add(len - 1) != b'/' {
            len -= 1;
        }

        if len > 0 {
            let tmp = *arg0.add(len);
            *arg0.add(len) = 0;
            let _res = crate::sys::io::sceIoChdir(arg0 as *const u8);
            *arg0.add(len) = tmp;
        }
    }
}

pub(crate) static MAIN_THREAD_ID: SyncUnsafeCell<Option<ThreadId>> = SyncUnsafeCell::new(None);
pub(crate) static CUSTOM_CLEANUP: SyncUnsafeCell<Option<fn()>> = SyncUnsafeCell::new(None);

pub(crate) fn main_thread_id() -> Option<ThreadId> {
    let id = unsafe { &*MAIN_THREAD_ID.get() };
    *id
}

pub(crate) fn custom_cleanup() -> Option<fn()> {
    let res = unsafe { &*CUSTOM_CLEANUP.get() };
    *res
}

/// Sets a custom cleanup procedure that will be called on [`exit`] and on the set callback by
/// [`enable_home_button`].
///
/// It returns the previously set value.
///
/// [`exit`]: crate::process::exit
/// [`enable_home_button`]: crate::enable_home_button
pub fn set_custom_cleanup(f: fn()) -> Option<fn()> {
    let cleanup = CUSTOM_CLEANUP.get();
    let previous = unsafe { cleanup.read() };
    unsafe {
        cleanup.write(Some(f));
    }

    previous
}
pub(crate) unsafe fn init(_argc: usize, argv: *const *mut u8) {
    // Ideally this is called on module_start, but people doing barebones project may forget, so we
    // call it here too.
    crate::set_psp_os_functions();

    let main_thread_id = sys::thread::sceKernelGetThreadId().into_result();

    if let Ok(id) = main_thread_id {
        unsafe { MAIN_THREAD_ID.get().write_volatile(Some(id)) };
    }

    unsafe {
        init_cwd(*argv);
    }

    // Enable
    if cfg!(pbp) {
        crate::enable_home_button();
    }
}

/// Cleanup procedure to be run after `main`
///
/// If you are a plugin, remember to do this manually
pub(crate) fn cleanup() {
    use crate::sync::nonpoison::Once;

    static CLEANUP: Once = Once::new();
    CLEANUP.call_once(|| unsafe {
        // Flush stdout and disable buffering.
        crate::io::cleanup();
        // SAFETY: Only called once during runtime cleanup.
        sys::cleanup();

        if let Some(f) = custom_cleanup() {
            f();
        }
    });
}

fn handle_rt_panic<T>(e: alloc::boxed::Box<dyn core::any::Any + Send>) -> T {
    core::mem::forget(e);
    cfg_select! {
        panic = "immediate-abort" => {}
        _ => {
            if let Some(mut out) = crate::os::stdio::panic_output() {
                let _ = crate::io::Write::write_fmt(
                    &mut out,
                    format_args!(
                        "fatal runtime error: {}, aborting\n",
                        format_args!("drop of the panic payload panicked")
                    ),
                );
            }
        }
    }
    crate::process::abort()
}

// To reduce the generated code of the new `psp_start`, this function is doing
// the real work.
#[cfg(not(test))]
fn psp_start_internal(
    main: &(dyn Fn() -> i32 + Sync + core::panic::RefUnwindSafe), argc: usize, argv: *const *mut u8,
) -> isize {
    // Guard against the code called by this function from unwinding outside of the Rust-controlled
    // code, which is UB. This is a requirement imposed by a combination of how the
    // `#[lang="start"]` attribute is implemented as well as by the implementation of the panicking
    // mechanism itself.
    //
    // There are a couple of instances where unwinding can begin. First is inside of the
    // `rt::init`, `rt::cleanup` and similar functions controlled by std. In those instances a
    // panic is a std implementation bug. A quite likely one too, as there isn't any way to
    // prevent std from accidentally introducing a panic to these functions. Another is from
    // user code from `main` or, more nefariously, as described in e.g. issue #86030.
    //
    // We use `catch_unwind` with `handle_rt_panic` instead of `abort_unwind` to make the error in
    // case of a panic a bit nicer.
    panicking::catch_unwind(move || {
        // SAFETY: Only called once during runtime initialization.
        unsafe { init(argc, argv) };

        let ret_code = panicking::catch_unwind(main).unwrap_or_else(move |payload| {
            // Carefully dispose of the panic payload.
            let payload = panicking::AssertUnwindSafe(payload);
            panicking::catch_unwind(move || drop({ payload }.0)).unwrap_or_else(move |e| {
                core::mem::forget(e); // do *not* drop the 2nd payload
                cfg_select! {
                    panic = "immediate-abort" => {}
                    _ => {
                        if let Some(mut out) = crate::os::stdio::panic_output() {
                            let _ = crate::io::Write::write_fmt(
                                &mut out,
                                format_args!(
                                    "fatal runtime error: {}, aborting\n",
                                    format_args!("drop of the panic payload panicked")
                                ),
                            );
                        }
                    }
                }

                crate::process::abort()
            });
            // Return error code for panicking programs.
            101
        });
        let ret_code = ret_code as isize;

        cleanup();

        // Guard against multiple threads calling `libc::exit` concurrently.
        // See the documentation for `unique_thread_exit` for more information.
        // crate::sys::exit::unique_thread_exit();

        ret_code
    })
    .unwrap_or_else(handle_rt_panic)
}

// WARNING: `argc_bytes` and `argp` are not the same as `argc` and `argv`.
//
// The PSP system allows to pass arbitrary data to the `sceKernelCreateThread` function pointer.
// It can be a struct reference, and array of data, etc.
//
// It just happen that for module start the data passed is passed as a form similar to `argc` and
// `argv`, but it needs to be processed before.
//
// In this regard, this is different from `rust_start`
#[cfg(not(any(test, doctest)))]
#[doc(hidden)]
pub fn psp_start<T: crate::process::Termination + 'static>(
    main: fn() -> T, argc: usize, argv: *const *mut u8,
) -> isize {
    psp_start_internal(&move || main().report().to_i32(), argc, argv)
}

//! Runtime services
//!
//! The `rt` module provides a narrow set of runtime services,
//! including the global heap (exported in `heap`) and unwinding and
//! backtrace support. The APIs in this module are highly unstable,
//! and should be considered as private implementation details for the
//! time being.

use crate::{panic, sys};

const MAX_ARGC: usize = 19;

/// Process `argc_bytes` and `argp` from `module_start` and creates a `argc` and `argv` to pass to
/// be passed to [`sceKernelStartThread`]. when starting the module main thread.
///
/// [`sceKernelStartThread`]: crate::sys::thread::sceKernelStartThread
///
/// # Safety
/// `argp` must be valid for `argc_bytes`. And this function is only to be used with `module_start`.
pub unsafe fn process_argc_argv(
    argc_bytes: usize, argp: *const core::ffi::c_void,
) -> (usize, [*const core::ffi::c_char; MAX_ARGC + 1]) {
    let mut argv: [*const core::ffi::c_char; 20] = [core::ptr::null(); 20];
    let mut argc = 0;
    let mut loc = 0;
    let ptr: *const core::ffi::c_char = argp.cast();

    while loc < argc_bytes {
        unsafe {
            argv[argc] = ptr.add(loc) as *const core::ffi::c_char;

            let arg_len = crate::private::strlen(argv[argc]) + 1;

            loc += arg_len;
            argc += 1;

            if argc == 19 {
                break;
            }
        }
    }

    (argc, argv)
}

/// Cleanup procedure to be run after `main`
///
/// If you are a plugin, remember to do this manually
#[cfg(feature = "non-stub-code")]
fn cleanup() {
    use crate::sync::nonpoison::Once;

    static CLEANUP: Once = Once::new();
    CLEANUP.call_once(|| unsafe {
        // Flush stdout and disable buffering.
        crate::io::cleanup();
        // SAFETY: Only called once during runtime cleanup.
        sys::cleanup();
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
    // FIXME ABORT HERE
    todo!()
}

// To reduce the generated code of the new `psp_start`, this function is doing
// the real work.
#[cfg(not(test))]
fn psp_start_internal(
    main: &(dyn Fn() -> i32 + Sync + core::panic::RefUnwindSafe), _argc: isize,
    _argv: *const *const u8,
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
    panic::catch_unwind(move || {
        // SAFETY: Only called once during runtime initialization.
        // unsafe { init(argc, argv) };

        let ret_code = panic::catch_unwind(main).unwrap_or_else(move |payload| {
            // Carefully dispose of the panic payload.
            let payload = panic::AssertUnwindSafe(payload);
            panic::catch_unwind(move || drop({ payload }.0)).unwrap_or_else(move |e| {
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

                // FIXME ABORT HERE
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

#[cfg(not(any(test, doctest)))]
#[doc(hidden)]
pub fn psp_start<T: crate::termination::Termination + 'static>(
    main: fn() -> T, argc: isize, argv: *const *const u8,
) -> isize {
    psp_start_internal(&move || main().report().to_i32(), argc, argv)
}

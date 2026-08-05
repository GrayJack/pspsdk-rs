/// Prints to the PSP screen.
///
/// Equivalent to the [`dprintln!`] macro except that a newline is not printed at
/// the end of the message.
///
/// Note that the macro is intended as a debugging tool and therefore you
/// should avoid having uses of it in version control for long periods
/// (other than in tests and similar).
///
/// [`dprintln!`]: crate::dprintln
#[macro_export]
#[cfg(feature = "non-stub-code")]
macro_rules! dprint {
    ($($arg:tt)*) => {{
        $crate::io::printing::_dprint(::core::format_args!($($arg)*));
    }};
}

/// Prints to the PSP screen, with a newline.
///
/// Note that the macro is intended as a debugging tool and therefore you
/// should avoid having uses of it in version control for long periods
/// (other than in tests and similar).
#[macro_export]
#[cfg(feature = "non-stub-code")]
macro_rules! dprintln {
    () => {
        $crate::dprint!("\n")
    };
    ($($arg:tt)*) => {{
        $crate::dprint!("{}\n", ::core::format_args!($($arg)*));
    }};
}

/// Prints to the PSP standard output.
///
/// Equivalent to the [`println!`] macro except that a newline is not printed at
/// the end of the message.
///
/// Note that stdout is frequently line-buffered by default so it may be
/// necessary to use [`io::stdout().flush()`][flush] to ensure the output is emitted
/// immediately.
///
/// The `print!` macro will lock the standard output on each call. If you call
/// `print!` within a hot loop, this behavior may be the bottleneck of the loop.
/// To avoid this, lock stdout with [`io::stdout().lock()`][lock]:
/// ```
/// use pspsdk::io::{stdout, Write};
///
/// let mut lock = stdout().lock();
/// write!(lock, "hello world").unwrap();
/// ```
///
/// Use `print!` only for the primary output of your program. Use
/// [`eprint!`] instead to print error and progress messages.
///
/// [flush]: crate::io::Write::flush
/// [`println!`]: crate::println
/// [`eprint!`]: crate::eprint
/// [lock]: crate::io::Stdout
///
/// # Panics
///
/// Panics if writing to `io::stdout()` fails.
///
/// Writing to non-blocking stdout can cause an error, which will lead
/// this macro to panic.
///
/// # Examples
///
/// ```
/// use pspsdk::{
///     io::{self, Write},
///     print,
/// };
///
/// print!("this ");
/// print!("will ");
/// print!("be ");
/// print!("on ");
/// print!("the ");
/// print!("same ");
/// print!("line ");
///
/// io::stdout_raw().flush().unwrap();
///
/// print!("this string has a newline, why not choose println! instead?\n");
///
/// io::stdout_raw().flush().unwrap();
/// ```
#[macro_export]
#[cfg(feature = "non-stub-code")]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::io::printing::_print(::core::format_args!($($arg)*));
    }};
}

/// Prints to the PSP standard output, with a newline.
///
/// On all platforms, the newline is the LINE FEED character (`\n`/`U+000A`) alone
/// (no additional CARRIAGE RETURN (`\r`/`U+000D`)).
///
/// This macro uses the same syntax as [`format!`], but writes to the standard output instead.
/// See [`core::fmt`] for more information.
///
/// The `println!` macro will lock the standard output on each call. If you call
/// `println!` within a hot loop, this behavior may be the bottleneck of the loop.
/// To avoid this, lock stdout with [`io::stdout().lock()`][lock]:
/// ```
/// use pspsdk::io::{stdout, Write};
///
/// let mut lock = stdout().lock();
/// writeln!(lock, "hello world").unwrap();
/// ```
///
/// Use `println!` only for the primary output of your program. Use
/// [`eprintln!`] instead to print error and progress messages.
///
/// [`core::fmt`]: core::fmt
/// [`eprintln!`]: crate::eprintln
/// [`format!`]: alloc::format
/// [lock]: crate::io::Stdout
///
/// # Panics
///
/// Panics if writing to [`io::stdout`] fails.
///
/// Writing to non-blocking stdout can cause an error, which will lead
/// this macro to panic.
///
/// [`io::stdout`]: crate::io::stdout
///
/// # Examples
///
/// ```
/// pspsdk::println!(); // prints just a newline
/// pspsdk::println!("hello there!");
/// pspsdk::println!("format {} arguments", "some");
/// let local_variable = "some";
/// pspsdk::println!("format {local_variable} arguments");
/// ```
#[macro_export]
#[cfg(feature = "non-stub-code")]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {{
        $crate::print!("{}\n", ::core::format_args!($($arg)*));
    }};
}

/// Prints to the PSP standard error.
/// Equivalent to the [`print!`] macro, except that output goes to
/// [`io::stderr`] instead of [`io::stdout`]. See [`print!`] for
/// example usage.
///
/// Use `eprint!` only for error and progress messages. Use `print!`
/// instead for the primary output of your program.
///
/// [`io::stderr`]: crate::io::stderr
/// [`io::stdout`]: crate::io::stdout
///
/// # Panics
///
/// Panics if writing to `io::stderr` fails.
///
/// Writing to non-blocking stderr can cause an error, which will lead
/// this macro to panic.
///
/// # Examples
///
/// ```
/// pspsdk::eprint!("Error: Could not complete task");
/// ```
#[macro_export]
#[cfg(feature = "non-stub-code")]
macro_rules! eprint {
    ($($arg:tt)*) => {{
        $crate::io::printing::_eprint(::core::format_args!($($arg)*));
    }};
}

/// Prints to the PSP standard error, with a newline.
///
/// Equivalent to the [`println!`] macro, except that output goes to
/// [`io::stderr`] instead of [`io::stdout`]. See [`println!`] for
/// example usage.
///
/// Use `eprintln!` only for error and progress messages. Use `println!`
/// instead for the primary output of your program.
///
/// [`io::stderr`]: crate::io::stderr
/// [`io::stdout`]: crate::io::stdout
/// [`println!`]: crate::println
///
/// # Panics
///
/// Panics if writing to `io::stderr` fails.
///
/// Writing to non-blocking stderr can cause an error, which will lead
/// this macro to panic.
///
/// # Examples
///
/// ```
/// pspsdk::eprintln!("Error: Could not complete task");
/// ```
#[macro_export]
#[cfg(feature = "non-stub-code")]
macro_rules! eprintln {
    () => {
        $crate::eprint!("\n")
    };
    ($($arg:tt)*) => {{
        $crate::eprint!("{}\n", ::core::format_args!($($arg)*));
    }};
}

/// Prints and returns the value of a given expression for quick and dirty
/// debugging.
///
/// An example:
///
/// ```rust
/// let a = 2;
/// let b = dbg!(a * 2) + 1;
/// //      ^-- prints: [src/main.rs:2:9] a * 2 = 4
/// assert_eq!(b, 5);
/// ```
///
/// The macro works by using the `Debug` implementation of the type of
/// the given expression to print the value to [stderr] along with the
/// source location of the macro invocation as well as the source code
/// of the expression.
///
/// Invoking the macro on an expression moves and takes ownership of it
/// before returning the evaluated expression unchanged. If the type
/// of the expression does not implement `Copy` and you don't want
/// to give up ownership, you can instead borrow with `dbg!(&expr)`
/// for some expression `expr`.
///
/// The `dbg!` macro works exactly the same in release builds.
/// This is useful when debugging issues that only occur in release
/// builds or when debugging in release mode is significantly faster.
///
/// Note that the macro is intended as a debugging tool and therefore you
/// should avoid having uses of it in version control for long periods
/// (other than in tests and similar).
///
/// # Stability
///
/// The exact output printed by this macro should not be relied upon
/// and is subject to future changes.
///
/// # Panics
///
/// Panics if writing to `io::stderr` fails.
///
/// # Further examples
///
/// With a method call:
///
/// ```rust
/// use pspsdk::dbg;
///
/// fn foo(n: usize) {
///     if let Some(_) = dbg!(n.checked_sub(4)) {
///         // ...
///     }
/// }
///
/// foo(3)
/// ```
///
/// This prints to [stderr]:
///
/// ```text,ignore
/// [src/main.rs:2:22] n.checked_sub(4) = None
/// ```
///
/// Naive factorial implementation:
///
/// ```rust
/// use pspsdk::dbg;
///
/// fn factorial(n: u32) -> u32 {
///     if dbg!(n <= 1) {
///         dbg!(1)
///     } else {
///         dbg!(n * factorial(n - 1))
///     }
/// }
///
/// dbg!(factorial(4));
/// ```
///
/// This prints to [stderr]:
///
/// ```text,ignore
/// [src/main.rs:2:8] n <= 1 = false
/// [src/main.rs:2:8] n <= 1 = false
/// [src/main.rs:2:8] n <= 1 = false
/// [src/main.rs:2:8] n <= 1 = true
/// [src/main.rs:3:9] 1 = 1
/// [src/main.rs:7:9] n * factorial(n - 1) = 2
/// [src/main.rs:7:9] n * factorial(n - 1) = 6
/// [src/main.rs:7:9] n * factorial(n - 1) = 24
/// [src/main.rs:9:1] factorial(4) = 24
/// ```
///
/// The `dbg!(..)` macro moves the input:
///
/// ```compile_fail
/// use pspsdk::dbg;
///
/// /// A wrapper around `usize` which importantly is not Copyable.
/// #[derive(Debug)]
/// struct NoCopy(usize);
///
/// let a = NoCopy(42);
/// let _ = dbg!(a); // <-- `a` is moved here.
/// let _ = dbg!(a); // <-- `a` is moved again; error!
/// ```
///
/// You can also use `dbg!()` without a value to just print the
/// file and line whenever it's reached.
///
/// Finally, if you want to `dbg!(..)` multiple values, it will treat them as
/// a tuple (and return it, too):
///
/// ```
/// assert_eq!(pspsdk::dbg!(1usize, 2u32), (1, 2));
/// ```
///
/// However, a single argument with a trailing comma will still not be treated
/// as a tuple, following the convention of ignoring trailing commas in macro
/// invocations. You can use a 1-tuple directly if you need one:
///
/// ```
/// assert_eq!(1, pspsdk::dbg!(1u32,)); // trailing comma ignored
/// assert_eq!((1,), pspsdk::dbg!((1u32,))); // 1-tuple
/// ```
///
/// [stderr]: https://en.wikipedia.org/wiki/Standard_streams#Standard_error_(stderr)
#[macro_export]
#[cfg(feature = "non-stub-code")]
macro_rules! dbg {
    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `eprintln!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `eprintln!`
    // will be malformed.
    () => {
        $crate::eprintln!("[{}:{}:{}]", ::core::file!(), ::core::line!(), ::core::column!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::eprintln!("[{}:{}:{}] {} = {:#?}",
                    ::core::file!(),
                    ::core::line!(),
                    ::core::column!(),
                    ::core::stringify!($val),
                    // The `&T: Debug` check happens here (not in the format literal desugaring)
                    // to avoid format literal related messages and suggestions.
                    &&tmp as &dyn ::core::fmt::Debug,
                );
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

/// Call the `psp_main` function. handling things on ffi boundary.
///
/// # Syntax
///
/// ```no_run
/// call_main!(path::to::psp_main, argc_value, argv_ptr);
/// ```
///
/// # Example
///
/// ```no_run
/// extern "C" fn psp_main_thread(argc: usize, argv: *mut c_void) -> SceResult<u32> {
///     let res = pspsdk::call_main!(psp_main, argc, argv);
///
///     pspsdk::process::exit(res as i32);
/// }
///
/// fn psp_main() {}
/// ```
#[macro_export]
#[cfg(feature = "non-stub-code")]
macro_rules! call_main {
    ($psp_main:expr, $argc_bytes:expr, $argp:expr) => {{
        $crate::psp_start($psp_main, $argc_bytes, (&raw const $argp).cast())
    }};
}


// Because code generated by `module!()` lives in user's crate, cfg attribute cannot be used there.
// Hence this macro.
#[cfg(feature = "std")]
#[cfg(feature = "non-stub-code")]
#[doc(hidden)]
#[macro_export]
macro_rules! _start {
    ($_:expr, $argc:expr, $argv:expr) => {
        unsafe { ::core::mem::transmute(unsafe { $crate::c_main($argc as _, $argv as _) as _ }) }
    };
}
#[cfg(not(feature = "std"))]
#[cfg(feature = "non-stub-code")]
#[doc(hidden)]
#[macro_export]
macro_rules! _start {
    ($psp_main:expr, $argc:expr, $argv:expr) => {{
        let res = $crate::psp_start($psp_main, $argc, (&raw const $argv).cast());

        pspsdk::process::exit(res as i32);
        $crate::sys::SceResult::new(0)
    }};
}

/// Declare a PSP module info.
///
/// This **does not** include basic `syslib` export like [`module!`].
///
/// [`module!`]: crate::module
///
/// # Syntax
/// ```no_run
/// module_info!(<module_name>, <major_version>, <minor_version>, [module_attribute]);
/// ```
///
/// # Example
///
/// ```no_run
/// pspsdk::module_info!("MyModule", 1, 0);
/// ```
///
/// With attributes
///
/// ```no_run
/// use pspsdk::sys::module::ModuleAttributes;
///
/// pspsdk::module_info!("MyModule", 1, 0, ModuleAttributes::User);
/// ```
#[macro_export]
macro_rules! module_info {
    ($name:expr, $version_major:expr, $version_minor:expr) => {
        $crate::module_info!(
            $name,
            $version_major,
            $version_minor,
            $crate::sys::library::ModuleAttributes::base_default()
        );
    };

    ($name:expr, $version_major:expr, $version_minor:expr, $attr:expr) => {
        #[used]
        #[unsafe(no_mangle)]
        #[unsafe(link_section = ".rodata.sceModuleInfo")]
        static module_info: $crate::Align16<$crate::sys::library::ModuleInfo> =
            $crate::Align16($crate::sys::library::ModuleInfo {
                attributes: $attr,
                version: ($version_major, $version_minor),
                name: $crate::sys::library::ModuleInfo::name_from_str($name),
                terminal_char: b'\0',
                gp: unsafe { &_gp },
                stub_top: unsafe { &__lib_stub_top },
                stub_end: unsafe { &__lib_stub_bottom },
                entry_top: unsafe { &__lib_ent_top },
                entry_end: unsafe { &__lib_ent_bottom },
            });

        unsafe extern "C" {
            static _gp: u8;
            static __lib_ent_bottom: u8;
            static __lib_ent_top: u8;
            static __lib_stub_bottom: u8;
            static __lib_stub_top: u8;
        }
    };
}

/// Creates the default module_start implementation.
///
/// # Syntax
///
/// ```no_run
/// default_module_start!(path::to::psp::main);
/// ```
///
/// # Example
///
/// ```no_run
/// pspsdk::default_module_start!(psp_main);
/// ```
#[macro_export]
#[cfg(feature = "non-stub-code")]
macro_rules! default_module_start {
    ($psp_main:expr) => {
        #[unsafe(no_mangle)]
        extern "C" fn module_start(argc_bytes: usize, argp: *mut ::core::ffi::c_void) -> isize {
            #[allow(unreachable_code)]
            extern "C" fn main_thread(
                argc: usize, argv: *mut ::core::ffi::c_void,
            ) -> $crate::sys::SceResult<u32> {
                $crate::_start!($psp_main, argc, argv)
            }

            let (argc, mut argv) = unsafe { $crate::module_start_init(argc_bytes, argp.cast()) };

            unsafe {
                let Ok(id) = $crate::sys::thread::sceKernelCreateThread(
                    c"main_thread".as_ptr().cast(),
                    main_thread,
                    // default priority of 32.
                    32,
                    // 256kb stack
                    256 * 1024,
                    $crate::sys::thread::ThreadAttributes::main_default(),
                    None,
                )
                .into_result() else {
                    return -1;
                };

                let Ok(()) =
                    $crate::sys::thread::sceKernelStartThread(id, argc, argv.as_mut_ptr().cast())
                        .into_result()
                else {
                    return -1;
                };
            }

            0
        }
    };
}

/// Declare a PSP module.
///
/// You must also define a `fn psp_main() { ... }` function in conjunction with
/// this macro.
///
/// Compat with `psp-rs`
///
/// # Syntax
/// ```no_run
/// module!(<module_name>, <major_version>, <minor_version>);
/// ```
///
/// # Example
///
/// ```no_run
/// pspsdk::module!("ModuleName", 1, 0);
/// ```
#[macro_export]
#[cfg(feature = "non-stub-code")]
macro_rules! module {
    ($name:expr, $version_major:expr, $version_minor:expr) => {
        #[doc(hidden)]
        mod __psp_module {
            $crate::module_info!($name, $version_major, $version_minor);

            $crate::default_module_start!(super::psp_main);

            $crate::exports! {
                "syslib", $version_major, $version_minor, 0x8000, [
                    fn module_start,
                    static module_info.0 : 0xF01D73A7,
                ];
            }
        }
    };
}


// FIXME: Remove from exports and move

// Prints to the "panic output", depending on the platform this may be:
// - the standard error output
// - some dedicated platform specific output
// - nothing (so this macro is a no-op)
#[doc(hidden)]
#[macro_export]
macro_rules! rtprintpanic {
    ($($t:tt)*) => {
        #[cfg(not(panic = "immediate-abort"))] {
            $crate::panicking::print(format_args!($($t)*));
        }

        #[cfg(panic = "immediate-abort")]
        {
            let _ = format_args!($($t)*);
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! rtabort {
    ($($t:tt)*) => {
        {
            $crate::panicking::print_and_die(format_args!("fatal runtime error: {}, aborting\n", format_args!($($t)*)));
        }
    }
}

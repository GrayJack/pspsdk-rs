#![no_std]
#![no_main]

use core::ffi::c_void;

use pspsdk::sys::{
    thread::{sceKernelCreateThread, sceKernelStartThread, ThreadAttributes},
    SceResult,
};

pspsdk::module_info!("sample_module", 1, 1);

#[unsafe(no_mangle)]
extern "C" fn module_start(argc_bytes: usize, argp: *mut c_void) -> isize {
    // Set OS functions for pspsdk::io module
    pspsdk::set_psp_os_functions();

    // Set argc argv
    let (argc, argv) = unsafe { pspsdk::process_argc_argv(argc_bytes, argp) };

    // pspsdk::println!("Hello from module start! argc: {argc_bytes} argv:{:?}", argv);

    unsafe {
        let Ok(id) = sceKernelCreateThread(
            c"main_thread".as_ptr().cast(),
            psp_main_thread,
            32,
            256 * 1024,
            ThreadAttributes::empty(),
            None,
        )
        .inspect_err(|err| pspsdk::println!("{}", err.to_inner()))
        .into_result() else {
            return -1;
        };

        let Ok(()) = sceKernelStartThread(id, argc, argv.as_ptr().cast())
            .inspect_err(|err| pspsdk::println!("{}", err.to_inner()))
            .into_result()
        else {
            return -1;
        };
    }
    0
}

extern "C" fn psp_main_thread(argc: usize, argv: *const c_void) -> SceResult<u32> {
    let res = pspsdk::call_main!(psp_main, argc, argv);

    SceResult::new(res as u32)
}

fn psp_main() -> SceResult<()> {
    pspsdk::enable_home_button();

    pspsdk::println!("Hello PSP from rust!");

    pspsdk::sys::thread::sceKernelSleepThread()?;

    SceResult::OK
}

#[unsafe(no_mangle)]
extern "C" fn module_stop(_argc: usize, _argv: *const c_void) -> isize {
    0
}

pspsdk::exports! {
    "syslib", 0, 0, 0x8000, [
        fn module_start,
        fn module_stop,
        static module_info.0 : 0xF01D73A7,
    ];
}

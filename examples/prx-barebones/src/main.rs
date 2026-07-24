#![no_std]
#![no_main]

use core::ffi::c_void;

use pspsdk::sys::{
    thread::{sceKernelCreateThread, sceKernelStartThread, ThreadAttributes},
    SceResult,
};

pspsdk::module_info!("PrxExampleBare", 1, 1);

fn psp_main() -> SceResult<()> {
    pspsdk::println!("Hello PSP from rust!");

    pspsdk::sys::thread::sceKernelSleepThread()?;

    SceResult::OK
}

#[unsafe(no_mangle)]
extern "C" fn module_start(argc_bytes: usize, argp: *mut c_void) -> isize {
    #[allow(unreachable_code)]
    extern "C" fn psp_main_thread(argc: usize, argv: *mut c_void) -> SceResult<u32> {
        let res = pspsdk::call_main!(psp_main, argc, argv);

        pspsdk::process::exit(res as i32);
        SceResult::new(res as u32)
    }

    let (argc, mut argv) = unsafe { pspsdk::module_start_init(argc_bytes, argp) };

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

        let Ok(()) = sceKernelStartThread(id, argc, argv.as_mut_ptr().cast())
            .inspect_err(|err| pspsdk::println!("{}", err.to_inner()))
            .into_result()
        else {
            return -1;
        };
    }
    0
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

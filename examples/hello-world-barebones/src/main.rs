#![no_std]
#![no_main]

use core::ffi::c_void;

use pspsdk::sys::{
    module::ModuleAttributes,
    thread::{sceKernelCreateThread, sceKernelStartThread, ThreadAttributes, ThreadEntryFn},
    SceResult,
};

pspsdk::module_info!("HelloWorldExampBare", 1, 1);

fn psp_main() {
    pspsdk::enable_home_button();
    pspsdk::dprintln!("Hello PSP from rust!");
    pspsdk::println!("Hello PSP from rust!");
}

#[unsafe(no_mangle)]
extern "C" fn module_start(argc_bytes: usize, argp: *mut c_void) -> isize {
    extern "C" fn psp_main_thread(argc: usize, argv: *mut c_void) -> SceResult<u32> {
        let _res = pspsdk::call_main!(psp_main, argc, argv);

        // Use this to auto-exit once main is complete
        // pspsdk::process::exit(_res as i32);
        SceResult::new(0)
    }

    // Set OS functions for pspsdk::io module
    pspsdk::set_psp_os_functions();

    let (argc, mut argv) = unsafe { pspsdk::module_start_init(argc_bytes, argp) };

    let mut main_func: ThreadEntryFn = psp_main_thread;

    if module_info.0.attributes.contains(ModuleAttributes::Kernel) {
        // Make sure kernel modules has the kernel memory address
        main_func = unsafe { core::mem::transmute((main_func as usize) | 0x80000000) };
    }

    let thread_attr = ThreadAttributes::main_default();

    if thread_attr.contains(
        ThreadAttributes::UserMode | ThreadAttributes::UsbWlanMode | ThreadAttributes::VshMode,
    ) {
        // Make sure user threads does not have the kernel memory address
        main_func = unsafe { core::mem::transmute((main_func as usize) & 0x7FFFFFFF) };
    }

    unsafe {
        let Ok(id) = sceKernelCreateThread(
            c"main_thread".as_ptr().cast(),
            main_func,
            32,
            256 * 1024,
            thread_attr,
            None,
        )
        .inspect_err(|err| pspsdk::eprintln!("Create {:#X}", err.to_inner()))
        .into_result() else {
            return -1;
        };

        let res = sceKernelStartThread(id, argc, argv.as_mut_ptr().cast())
            .inspect_err(|err| pspsdk::eprintln!("Start {:#X}", err.to_inner()));

        if res.is_err() {
            return -1;
        }
    }
    0
}

#[unsafe(no_mangle)]
extern "C" fn module_stop(_argc: usize, _argv: *mut c_void) -> isize {
    0
}

pspsdk::exports! {
    "syslib", 0, 0, 0x8000, [
        fn module_start,
        fn module_stop,
        static module_info.0 : 0xF01D73A7,
    ];
}

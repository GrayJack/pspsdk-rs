#![feature(asm_experimental_arch)]
#![no_main]
#![no_std]

pspsdk::module!("VFPU Add Examples", 1, 1);

fn psp_main() {
    pspsdk::println!("Testing VFPU...");
    pspsdk::dprintln!("Testing VFPU...");

    let res = vfpu_add(123, 4);
    pspsdk::println!("VFPU 123 + 4 = {res}");
    pspsdk::dprintln!("VFPU 123 + 4 = {res}");
}

fn vfpu_add(a: i32, b: i32) -> i32 {
    let ret_val;

    unsafe {
        pspsdk::vfpu_asm! (
            // Convert `a` to float
            "mtc1 {a}, {ftmp}",
            "nop",
            "cvt.s.w {ftmp}, {ftmp}",
            "mfc1 {a}, {ftmp}",
            "nop",

            // Convert `b` to float
            "mtc1 {b}, {ftmp}",
            "nop",
            "cvt.s.w {ftmp}, {ftmp}",
            "mfc1 {b}, {ftmp}",
            "nop",

            // Perform addition
            "mtv {a}, S000",
            "mtv {b}, S001",
            "vadd.s S000, S000, S001",
            "mfv {ret}, S000",

            // Convert result to `i32`
            "mtc1 {ret}, {ftmp}",
            "nop",
            "cvt.w.s {ftmp}, {ftmp}",
            "mfc1 {ret}, {ftmp}",
            "nop",

            ftmp = out(freg) _,
            a = inout(reg) a => _,
            b = inout(reg) b => _,
            ret = out(reg) ret_val,
            options(nostack, nomem),
        );
    }

    ret_val
}

#![no_std]
#![no_main]

pspsdk::module!("HelloWorldExample", 1, 1);

fn psp_main() {
    pspsdk::dprintln!("Hello PSP from rust!");
    pspsdk::println!("Hello PSP from rust!");
}

#![no_std]
#![no_main]

pspsdk::module!("sample_module", 1, 1);

fn psp_main() {
    pspsdk::enable_home_button();

    pspsdk::dprintln!("Hello PSP from rust!");
    pspsdk::println!("Hello PSP from rust!");
}

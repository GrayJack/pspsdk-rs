use std::env;

const ALLOWED_CFGS: &[&str] = &["pbp", "eboot", "pboot", "prx", "os_err_human"];

const PSPSDK_TARGET_FW: Option<&str> = std::option_env!("PSPSDK_TARGET_FW");

fn main() {
    set_check_cfg();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=libunwind.a");
    println!("cargo:rerun-if-changed=libunwind_lto.a");
    println!("cargo:rerun-if-env-changed=RUSTFLAGS");

    let target_fw: Option<u32> = PSPSDK_TARGET_FW
        .map(|s| s.split('.').collect())
        .and_then(|s: String| s.parse::<u32>().ok());

    // Automatically set the kernel NID variant based on the targeted firmware.
    if let Some(target_fw) = target_fw {
        match target_fw {
            ..370 => println!("cargo:rustc-cfg=feature=\"psp_100\""),
            370..380 => println!("cargo:rustc-cfg=feature=\"psp_370\""),
            380..395 => println!("cargo:rustc-cfg=feature=\"psp_380\""),
            395..420 => println!("cargo:rustc-cfg=feature=\"psp_395\""),
            420..500 => println!("cargo:rustc-cfg=feature=\"psp_420\""),
            500..570 => println!("cargo:rustc-cfg=feature=\"psp_500\""),
            570..600 => println!("cargo:rustc-cfg=feature=\"psp_570\""),
            600..630 => println!("cargo:rustc-cfg=feature=\"psp_600\""),
            630..660 => println!("cargo:rustc-cfg=feature=\"psp_630\""),
            660..=661 => println!("cargo:rustc-cfg=feature=\"psp_660\""),
            _ => {},
        }
    }

    if env::var("CARGO_FEATURE_STUB_ONLY").is_ok() {
        return;
    }

    // Figure out whether to use the LTO libunwind, or the regular one.
    let unwind = if env::var("CARGO_ENCODED_RUSTFLAGS")
        .unwrap()
        .split('\x1f')
        .any(|flags| flags.starts_with("-Clinker-plugin-lto"))
    {
        "unwind_lto"
    } else {
        "unwind"
    };

    if cfg!(not(panic = "immediate-abort")) {
        println!("cargo:rustc-link-lib=static={unwind}");
        println!("cargo:rustc-link-search=native=./");
        println!("cargo:rustc-link-search=native=./pspsdk");
    }
}

fn set_check_cfg() {
    for cfg in ALLOWED_CFGS.iter() {
        println!("cargo::rustc-check-cfg=cfg({cfg})");
    }
}

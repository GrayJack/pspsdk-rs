use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=libunwind.a");
    println!("cargo:rerun-if-changed=libunwind_lto.a");
    println!("cargo:rerun-if-env-changed=RUSTFLAGS");

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

    println!("cargo:rustc-link-lib=static={unwind}");
    println!("cargo:rustc-link-search=native=./");
}

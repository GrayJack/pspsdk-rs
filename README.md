<h1 align="center">PSPSDK-RS</h1>

<p align="center">
    A work-in-progress crate for building PSP modules, including both PRX
    plugins and regular homebrew apps.
</p>

```rust
#![no_std]
#![no_main]

pspsdk::module!("ExampleModule", 1, 0);

fn psp_main() {
    pspsdk::println!("Hello PSP from rust!");
    pspsdk::dprintln!("Hello PSP from rust!");
}
```

## What about rust-psp?

This project is a new Rust SDK for PSP, architected from scratch, but some bits
and pieces from rust-sdk were used with some modifications to catalise the
initial process to minimum working project.

## What about PSPSDK?

This project is a completely new SDK, with no dependency on the original C/C++
PSPSDK. Like `rust-psp`, it aims to be a **complete** replacement, with more
efficient implementations of graphics functions, and the addition of missing
libraries, but also having a more type-safe FFI and high-level abstraction.

## Features / Roadmap

- [x] `core` support
- [x] `alloc` support
- [x] `panic = "unwind"` support
- [x] PSP system library support
- [x] No dependency on PSPSDK / PSPToolchain
- [x] Add support for creating kernel mode modules
- [x] Add support to export functions and static variables
- [ ] Macro-based VFPU assembler
- [ ] Full 3D graphics support
- [ ] Reach full parity with user mode support in PSPSDK
- [ ] Reach full parity with kernel mode support in PSPSDK
- [ ] Automatically sign EBOOT.PBP files to run on unmodified PSPs
- [ ] Implement / reverse undiscovered libraries
- [ ] Add `std` support

## Dependencies

To compile for the PSP, you will need a Rust **nightly** version equal to or
later than `2026-06-30` and the `rust-src` component. Please install Rust using
https://rustup.rs/

Use the following if you are new to Rust. (Feel free to set an override manually
per-project instead).

```sh
$ rustup default nightly && rustup component add rust-src
```

You also need `cargo-pspbuild` installed:

```sh
$ cargo install cargo-pspbuild
```

## Usage

To use the `pspsdk` crate in your own Rust programs, add it to `Cargo.toml` like
any other dependency:

```toml
[dependencies]
pspsdk = "x.y.z"
```

In your `main.rs` file, you need to setup a basic skeleton like so:

```rust
#![no_std]
#![no_main]

// Create a module named "sample_module" with version 1.0
pspsdk::module!("sample_module", 1, 0);

fn psp_main() {
    pspsdk::dprintln!("Hello PSP from rust!");
}
```

Now you can simply run `cargo pspbuild` to build your project; by default, it
builds a `EBOOT.PBP` file. You can also invoke `cargo pspbuild --release` to
create a release build.

You can further configure your PSP project by creating a `Psp.toml` file in the
root of your project. Once created, the `[project]` section and `kind`
configuration option are mandatory

If you would like to customize your EBOOT with e.g. an icon or new title, you
can add a `[project.pbp]` section on your `Psp.toml` file. Note that all
`[project.pbp]` keys are optional:

```toml
[project]
kind = "EBOOT"

[project.pbp]
title = "XMB title"
xmb_icon_png = "path/to/24bit_144x80_image.png"
xmb_background_png = "path/to/24bit_480x272_background.png"
xmb_music_at3 = "path/to/ATRAC3_audio.at3"
```

More options can be found in the schema definition [here](/cargo-pspbuild/src/main.rs#L18-L132).


## `error[E0460]: found possibly newer version of crate ...`

If you get an error like this:

```
error[E0460]: found possibly newer version of crate `panic_unwind` which `pspsdk` depends on
 --> src/main.rs:4:5
  |
4 | use pspsdk::dprintln;
  |     ^^^^^^
  |
  = note: perhaps that crate needs to be recompiled?
```

Simply clean your target directory and it will be fixed:

```sh
$ cargo clean
```
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, Result};

use crate::{
    export_impl::ExportArgs,
    exports_impl::ExportsArgs,
    pspfwcfg::{PspFwCfgArg, PspFwSelect, PspFwSelectArm},
    pspstub::{PspStub, StubArgs},
};

mod export_impl;
mod exports_impl;
mod pspfwcfg;
mod pspstub;

/// Creates a PSP stub library from a `extern "C"` block.
///
/// # Macro parameters
///
/// - `libname`: The name of the library to create stubs.
/// - `flags`: The library flags. It can be a integer literal or a expression of `LibFlags`.
/// - `version`: Optional. A tuple with a major and minor version of the stub library. If not set,
///   it defaults to `(0, 0)`.
///
/// # Extern "C" block items attributes
///
/// - `nid`: An attribute that specifies the NID of the extern item. If an item lacks it, it derives
///   the NID value from a hash method from the item name.
/// - `eabi`: Specifies for function items with 5 or more parameters to generate a function that
///   uses the MIPS eabi calling convention. If not used, such functions default to use the `o32`
///   MIPS calling convention which may not work depending on many factors.
///   - `i5`: Function with 5 parameters (up to 32 bit in size) using eabi calling convention.
///   - `i6`: Function with 6 parameters (up to 32 bit in size) using eabi calling convention.
///   - `i7`: Function with 7 parameters (up to 32 bit in size) using eabi calling convention.
///   - `i8`: Function with 8 parameters (up to 32 bit in size) using eabi calling convention.
///   - `i_ii_i_ri`: Function with the parameters in the shape of `(32bit, 64bit, 32bit) -> 32bit`
///     using eabi calling convention.
///   - `i_ii_i_rii`: Function with the parameters in the shape of `(32bit, 64bit, 32bit) -> 64bit`
///     using eabi calling convention.
///
/// # Examples
///
/// ```no_run
/// #[pspsdk::psp_stub(libname = "LibraryName", flags = 0x0001, version = (1, 0))]
/// unsafe extern "C" {
///     // The NID will be automatically decided by hash.
///     unsafe fn stub_function();
///
///     // The NID can also be specified.
///     #[nid(0xDEADCAFE)]
///     safe fn another_function(arg: i32) -> i32;
/// }
/// ```
///
/// Specifying eabi usage:
///
/// ```no_run
/// #[pspsdk::psp_stub(libname = "LibraryName", flags = 0x0001)]
/// unsafe extern "C" {
///     #[eabi(i6)]
///     unsafe fn stub_function(a1: i32, a2: *mut u8, a3: u8, a4: u16, a5: bool, a6: usize);
/// }
/// ```
///
/// Using `LibFlags`:
///
/// ```no_run
/// #[pspsdk::psp_stub(libname = "LibraryName", flags = LibFlags::AutoExport.union(LibFlags::WeakImport), version = (1, 0))]
/// unsafe extern "C" {
///     unsafe fn stub_function();
/// }
/// ```
#[proc_macro_attribute]
pub fn psp_stub(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut args: StubArgs = StubArgs::default();
    let args_parser = syn::meta::parser(|meta| args.parse(meta));
    parse_macro_input!(attr with args_parser);

    match psp_stub_impl(args, item) {
        Ok(ts) => ts,
        Err(err) => err.into_compile_error().into(),
    }
}

fn psp_stub_impl(args: StubArgs, item: TokenStream) -> Result<TokenStream> {
    let psp_stub = PspStub::parse(args, item)?;

    Ok(psp_stub.to_token_stream().into())
}

/// Creates a single library export entry for a PSP module.
///
/// **Note:** This macro does not produce ".lib.ent".
///
/// ## Example
///
/// ```rust
/// #[unsafe(no_mangle)]
/// static EXAMPLE: u32 = 0xBADBAD;
///
/// #[unsafe(no_mangle)]
/// extern "C" fn example() -> i32 {
///     42
/// }
///
/// #[unsafe(no_mangle)]
/// extern "C" fn example2() -> i32 {
///     44
/// }
///
/// pspsdk::export!("libname", 1, 0, 0, [
///     // You can pass just the identifier and the NID will be calculated.
///     fn example,
///     // You can also pass a NID value instead
///     fn example2 : 0xDEADDEAD,
///     // To export a global static, use the static keyword
///     static EXAMPLE,
/// ]);
/// ```
#[proc_macro]
pub fn export(input: TokenStream) -> TokenStream {
    let ExportArgs {
        libname,
        major_ver,
        minor_ver,
        flags,
        table,
    } = parse_macro_input!(input as ExportArgs);

    let nul_libname = if libname.value() == "syslib" {
        quote! { ::core::ptr::null() }
    } else {
        let mut name = libname.value();
        name.push('\0');
        let name = syn::LitStr::new(&name, libname.span());
        quote! {#name.as_ptr()}
    };


    let export_len = table.len() * 2;
    let mut nid_list = Vec::with_capacity(table.len());
    let mut item_list = Vec::with_capacity(table.len());
    let mut var_count: u8 = 0;
    let mut func_count: u16 = 0;
    for item in table {
        nid_list.push(item.nid);

        let ident = item.item;
        match item.kind {
            export_impl::EntryKind::Function => {
                func_count += 1;
                item_list.push(quote! {
                    ::pspsdk::sys::library::ResidentLibraryEntryItem::new_fn((#ident) as *const ())
                });
            },
            export_impl::EntryKind::Variable => {
                var_count += 1;
                item_list.push(quote! {
                    ::pspsdk::sys::library::ResidentLibraryEntryItem::new_var((&raw const (#ident)).cast())
                })
            },
        }
    }

    let flag = if let syn::Expr::Lit(syn::ExprLit { lit, .. }) = &flags {
        match lit {
            syn::Lit::Int(lit_int) => quote! {
                ::pspsdk::sys::LibFlags::from_bits_truncate(#lit_int)
            },
            _ => {
                return TokenStream::from(
                    syn::Error::new(flags.span(), "invalid literal, expected {integer}")
                        .to_compile_error(),
                )
            },
        }
    } else {
        quote! {
            #flags
        }
    };

    let export_ident =
        syn::Ident::new(&format!("__{}_exports", libname.value()), Span::mixed_site());
    let export_entry = syn::Ident::new(&format!("__{}_entry", libname.value()), Span::mixed_site());

    let ts = quote! {
        #[used]
        #[unsafe(no_mangle)]
        #[unsafe(link_section = ".rodata.sceResident")]
        static #export_ident: [::pspsdk::sys::library::ResidentLibraryEntryItem; #export_len] = [
            #(::pspsdk::sys::library::ResidentLibraryEntryItem::new_nid(#nid_list) ,)*
            #(#item_list ,)*
        ];

        const #export_entry: ::pspsdk::sys::library::ResidentLibraryEntry = ::pspsdk::sys::library::ResidentLibraryEntry {
            name: #nul_libname,
            version: (#major_ver, #minor_ver),
            flags: #flag,
            len: ::pspsdk::sys::library::RESI_LIBRARY_ENTRY_TABLE_NEW_LEN,
            var_exp_count: #var_count,
            func_exp_count: #func_count,
            entry_table: #export_ident.as_ptr(),
            unk1: 0,
            unk2: 0,
            unk3: 0,
        };
    };

    ts.into()
}

/// Creates one or more library export entry for a PSP module and ".lib.ent".
///
/// The syntax rules follows the same logic as [`export`] macro, but with multiple items separated
/// by `;`.
///
/// # Example
///
/// ```rust
/// pspsdk::module_info!("example_module", 1, 1);
///
/// #[unsafe(no_mangle)]
/// static EXAMPLE: u32 = 0xBADBAD;
///
/// #[unsafe(no_mangle)]
/// extern "C" fn example() -> i32 {
///     42
/// }
///
/// #[unsafe(no_mangle)]
/// extern "C" fn example2() -> i32 {
///     44
/// }
///
/// #[unsafe(no_mangle)]
/// extern "C" fn module_start(argc_bytes: usize, argv: *mut ::core::ffi::c_void) -> u32 {
///     0
/// }
///
/// pspsdk::exports! {
///     "syslib", 0, 0, 0x8000, [
///         fn module_start,
///         static module_info.0 : 0xF01D73A7,
///     ];
///     "libname", 1, 0, 0, [
///         fn example,
///         fn example2 : 0xDEADDEAD,
///         static EXAMPLE,
///     ];
/// }
/// ```
#[proc_macro]
pub fn exports(input: TokenStream) -> TokenStream {
    let ExportsArgs { exports } = parse_macro_input!(input as ExportsArgs);

    let num = exports.len();
    let mut export_entries = Vec::with_capacity(exports.len());
    let mut export_macros = Vec::with_capacity(exports.len());
    for export in exports {
        let ExportArgs {
            libname,
            major_ver,
            minor_ver,
            flags,
            table,
        } = export;

        export_entries.push(syn::Ident::new(
            &format!("__{}_entry", libname.value()),
            Span::mixed_site(),
        ));

        export_macros.push(quote! {
            ::pspsdk::export!(#libname, #major_ver, #minor_ver, #flags, [ #(#table ,)* ]);
        });
    }

    let ts = quote! {
        #(#export_macros)*

        #[used]
        #[unsafe(no_mangle)]
        #[unsafe(link_section = ".lib.ent")]
        static LIB_ENT: [::pspsdk::sys::library::ResidentLibraryEntry; #num] = [
           #(#export_entries ,)*
        ];
    };

    ts.into()
}

const PSPSDK_TARGET_FW: Option<&str> = std::option_env!("PSPSDK_TARGET_FW");

/// Conditionally includes the form to which it is attached based on a PSP Firmware version
/// targeted.
///
/// The macro takes either a decimal integer literal or a range of the same. The integer number is
/// the firmware version without the ".".
///
/// # Example
///
/// ```no_run
/// use pspsdk::{psp_fw_cfg, psp_stub};
///
/// #[psp_stub(libname = "SceWhatever", flags = 0x4001)]
/// unsafe extern "C" {
///     // Set for a specific version to only show up this item when targeting that specific version
///     #[psp_fw_cfg(570)]
///     fn sceWhateverFnOne() -> i32;
///
///     // Ranges can also be used. Implicit minimum is `100` and implicit maximum is `661`.
///     #[psp_fw_cfg(..150)]
///     fn sceWhateverFnTwo() -> i32;
///     #[psp_fw_cfg(150..)]
///     fn sceWhateverFnThree() -> i32;
///     #[psp_fw_cfg(150..=570)]
///     fn sceWhateverFnFour() -> i32;
/// }
/// ```
///
/// It can also be use to specify values for specific versions of the firmware
///
/// ```no_run
/// #[pspsdk::psp_fw_cfg(570)]
/// const MY_PATCH_OFFSET: usize = 0x2345;
/// #[pspsdk::psp_fw_cfg(600..630)]
/// const MY_PATCH_OFFSET: usize = 0x4356;
/// ```
#[proc_macro_attribute]
pub fn psp_fw_cfg(attr: TokenStream, item: TokenStream) -> TokenStream {
    match psp_fw_cfg_impl(attr, item) {
        Ok(ts) => ts,
        Err(err) => err.into_compile_error().into(),
    }
}

fn psp_fw_cfg_impl(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let arg: PspFwCfgArg = syn::parse(attr)?;

    let target_fw: Option<u32> = PSPSDK_TARGET_FW
        .map(|s| s.split('.').collect())
        .and_then(|s: String| s.parse::<u32>().ok());

    let Some(target_fw) = target_fw else {
        // If not set, always show item
        return Ok(item);
    };

    let res = match arg {
        PspFwCfgArg::Lit(lit) => {
            if lit == target_fw {
                item
            } else {
                TokenStream::new()
            }
        },
        PspFwCfgArg::Range(start, end, limits) => match limits {
            syn::RangeLimits::HalfOpen(_) => {
                if (start..end).contains(&target_fw) {
                    item
                } else {
                    TokenStream::new()
                }
            },
            syn::RangeLimits::Closed(_) => {
                if (start..=end).contains(&target_fw) {
                    item
                } else {
                    TokenStream::new()
                }
            },
        },
    };

    Ok(res)
}

/// Evaluates to a boolean a targeted PSP firmware version match the expected version.
///
/// # Examples
///
/// ```no_run
/// use pspsdk::psp_fw;
///
/// const MY_PATCH_OFFSET: usize = if psp_fw!(400..500) {
///     0x4356
/// } else if psp_fw!(600..) {
///     0x2345
/// } else {
///     panic!("not a supported PSP firmware version")
/// };
/// ```
#[proc_macro]
pub fn psp_fw(input: TokenStream) -> TokenStream {
    let arg: PspFwCfgArg = parse_macro_input!(input as PspFwCfgArg);

    let target_fw: Option<u32> = PSPSDK_TARGET_FW
        .map(|s| s.split('.').collect())
        .and_then(|s: String| s.parse::<u32>().ok());

    let Some(target_fw) = target_fw else {
        // If not set, always false
        return quote! {false}.into();
    };


    match arg {
        PspFwCfgArg::Lit(lit) => {
            if lit == target_fw {
                quote! {true}.into()
            } else {
                quote! {false}.into()
            }
        },
        PspFwCfgArg::Range(start, end, limits) => match limits {
            syn::RangeLimits::HalfOpen(_) => {
                if (start..end).contains(&target_fw) {
                    quote! {true}.into()
                } else {
                    quote! {false}.into()
                }
            },
            syn::RangeLimits::Closed(_) => {
                if (start..=end).contains(&target_fw) {
                    quote! {true}.into()
                } else {
                    quote! {false}.into()
                }
            },
        },
    }
}

/// Selects code at compile-time based on `psp_fw` predicates.
///
/// This macro evaluates, at compile-time, a series of `psp_fw` predicates,
/// selects the first that is true, and emits the code guarded by that
/// predicate. The code guarded by other predicates is not emitted.
///
/// An optional trailing `_` wildcard can be used to specify a fallback. If
/// none of the predicates are true, a [`compile_error`] is emitted.
///
/// # Examples
///
/// ```no_run
/// pspsdk::psp_fw_select! {
///     ..150 => {
///         pub fn my_func() -> i32 { /* PSP <1.50 specific functionality */ }
///     },
///     150..370 => {
///         pub fn my_func() -> i32 { /* PSP 1.50~370 specific functionality */ }
///     },
///     _ => {
///         pub fn my_func() -> i32 { /* fallback implementation */ }
///     },
/// }
/// ```
///
/// The `psp_fw_select!` macro can also be used in expression position, with or without braces on
/// the right-hand side:
///
/// ```no_run
/// let _some_string = pspsdk::psp_fw_select! {
///     ..420 => "With great power comes great electricity bills",
///     _ => { "Behind every successful diet is an unwatched pizza" }
/// };
/// ```
///
/// It can also be used on type expression right-hand side:
///
/// ```no_run
/// struct StructV1 {
///     // fields
/// }
/// struct StructV2 {
///     // fields
/// }
///
/// type Struct = pspsdk::psp_fw_select! {
///     ..150 => StructV1,
///     _ => StructV2,
/// };
/// ```
#[proc_macro]
pub fn psp_fw_select(input: TokenStream) -> TokenStream {
    let arg: PspFwSelect = parse_macro_input!(input as PspFwSelect);

    match psp_fw_select_impl(arg) {
        Ok(ts) => ts,
        Err(err) => err.into_compile_error().into(),
    }
}

fn psp_fw_select_impl(arg: PspFwSelect) -> Result<TokenStream> {
    let target_fw: Option<u32> = PSPSDK_TARGET_FW
        .map(|s| s.split('.').collect())
        .and_then(|s: String| s.parse::<u32>().ok());

    let last_idx = arg.arms.len() - 1;

    let mut res = None;
    if let Some(target_fw) = target_fw {
        for (idx, PspFwSelectArm { cond, ts, .. }) in arg.arms.into_iter().enumerate() {
            match cond {
                pspfwcfg::PspFwSelectCondPat::Lit(lit) => {
                    if lit == target_fw {
                        res = Some(ts);
                    } else {
                        continue;
                    }
                },
                pspfwcfg::PspFwSelectCondPat::Range(start, end, limits) => match limits {
                    syn::RangeLimits::HalfOpen(_) => {
                        if (start..end).contains(&target_fw) {
                            res = Some(ts);
                        } else {
                            continue;
                        }
                    },
                    syn::RangeLimits::Closed(_) => {
                        if (start..=end).contains(&target_fw) {
                            res = Some(ts);
                        } else {
                            continue;
                        }
                    },
                },
                pspfwcfg::PspFwSelectCondPat::Wild => {
                    if idx != last_idx {
                        return Err(syn::Error::new(
                            Span::mixed_site(),
                            "wild pattern must always be the last arm",
                        ));
                    } else {
                        res = Some(ts);
                    }
                },
            }
        }
    }

    res.ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            "none of the predicates in this `psp_fw_select` evaluated to true",
        )
    })
}

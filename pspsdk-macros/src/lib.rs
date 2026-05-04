use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, Result};

use crate::{
    export_impl::ExportArgs,
    exports_impl::ExportsArgs,
    pspstub::{PspStub, StubArgs},
};

mod export_impl;
mod exports_impl;
mod pspstub;

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
            len: 4,
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

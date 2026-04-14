use proc_macro2::Span;
use quote::{quote, ToTokens};
use sha1::{Digest, Sha1};
use syn::{
    meta::ParseNestedMeta, parse::Parse, spanned::Spanned, Error, Expr, ExprLit, ExprTuple,
    ForeignItem, ForeignItemFn, Ident, ItemForeignMod, Lit, LitInt, LitStr, Meta, Signature,
};

pub struct PspStub {
    lib_info: LibInfo,
    items: Vec<PspExternItemInfo>,
    use_crate: bool,
}

impl PspStub {
    pub fn parse(args: StubArgs, item: proc_macro::TokenStream) -> syn::Result<Self> {
        if let (Some(libname), Some(flags)) = (args.lib_name, args.flags) {
            let extern_block: ItemForeignMod = syn::parse(item)?;

            let items = extern_block
            .items
            // .clone()
            .iter()
            .map(|item| syn::parse2(item.to_token_stream()))
            .collect::<Result<_, _>>()?;

            Ok(PspStub {
                lib_info: LibInfo {
                    lib_name: libname,
                    flags,
                    major_version: args.major_version,
                    minor_version: args.minor_version,
                },
                items,
                use_crate: args.use_crate,
            })
        } else {
            Err(Error::new(
                Span::mixed_site(),
                "`libname` and `flags` are required arguments",
            ))
        }
    }
}

impl ToTokens for PspStub {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            lib_info:
                LibInfo {
                    lib_name,
                    flags,
                    major_version,
                    minor_version,
                },
            items,
            use_crate,
        } = self;

        let crate_path = if *use_crate {
            quote! {crate}
        } else {
            quote! {::pspsdk}
        };

        let mod_name = Ident::new(&format!("__{}", lib_name.value()), tokens.span());
        let resident_var = Ident::new(&format!("__{}_RESIDENT", lib_name.value()), tokens.span());
        let resident_section =
            LitStr::new(&format!(".rodata.sceResident.{}", lib_name.value()), tokens.span());

        let nid_start_var = Ident::new(&format!("__{}_NID_START", lib_name.value()), tokens.span());
        let nid_start_section =
            LitStr::new(&format!(".rodata.sceNid.{}", lib_name.value()), tokens.span());

        let fn_stub_start_var =
            Ident::new(&format!("__{}_FNSTUB_START", lib_name.value()), tokens.span());
        let fn_stub_start_section =
            LitStr::new(&format!(".sceStub.text.{}", lib_name.value()), tokens.span());

        let var_stub_start_var =
            Ident::new(&format!("__{}_VARSTUB_START", lib_name.value()), tokens.span());
        let var_stub_start_section =
            LitStr::new(&format!(".rodata.sceVstub.{}", lib_name.value()), tokens.span());

        let stub_entry_var = Ident::new(&format!("__{}_STUB", lib_name.value()), tokens.span());
        let stub_entry_section =
            LitStr::new(&format!(".lib.stub.entry.{}", lib_name.value()), tokens.span());

        let var_stub_count = items.iter().filter(|e| e.is_static()).count() as u8;
        let fn_stub_count = items.iter().filter(|e| e.is_func()).count() as u16;

        let mut fn_stub_code = Vec::new();
        let mut fn_stub_extern_items = Vec::new();
        let mut fn_eabi_defs = Vec::new();

        for fn_stub in items.iter().filter(|e| e.is_func()) {
            let PspExternItemInfo { item, nid, eabi } = fn_stub;

            if let ForeignItem::Fn(ForeignItemFn {
                attrs, vis, sig, ..
            }) = item
            {
                let Signature { unsafety, .. } = sig;

                let cfg_attrs: Vec<_> =
                    attrs.iter().filter(|attr| attr.path().is_ident("cfg")).collect();

                let attrs: Vec<_> = attrs
                    .iter()
                    .filter(|attr| {
                        !attr.meta.path().is_ident("eabi") && !attr.meta.path().is_ident("nid")
                    })
                    .collect();

                let fn_nid_section = LitStr::new(
                    &format!(".rodata.sceNid.{}.{}", lib_name.value(), sig.ident),
                    tokens.span(),
                );
                let fn_nid_var = Ident::new(&format!("__{}_NID", sig.ident), tokens.span());
                let fn_stub_section = LitStr::new(
                    &format!(".sceStub.text.{}.{}", lib_name.value(), sig.ident),
                    tokens.span(),
                );
                let fn_stub_var = Ident::new(&format!("__{}_stub", sig.ident), tokens.span());

                let mut inner_sig = sig.clone();
                inner_sig.ident = fn_stub_var.clone();

                let fn_libname_link = LitStr::new(&fn_stub_var.to_string(), tokens.span());

                let stub_vars = quote! {
                    #(#cfg_attrs)*
                    #[unsafe(link_section = #fn_nid_section)]
                    static #fn_nid_var: u32 = #nid;

                    #(#cfg_attrs)*
                    #[unsafe(link_section = #fn_stub_section)]
                    #[unsafe(no_mangle)]
                    static #fn_stub_var: #crate_path::sys::macro_helpers::Stub = #crate_path::sys::macro_helpers::Stub {
                        lib_addr: &#stub_entry_var,
                        nid_addr: &#fn_nid_var
                    };

                };


                let stub_extern = match eabi.clone() {
                    Some(EabiAttr {
                        eabi: EabiKind::Arg5,
                        ..
                    }) => quote! {
                        #(#cfg_attrs)*
                        #[doc(hidden)]
                        #vis(crate) unsafe fn #fn_stub_var(_: u32, _: u32, _: u32, _: u32, _: u32) -> u32
                    },
                    Some(EabiAttr {
                        eabi: EabiKind::Arg6,
                        ..
                    }) => quote! {
                        #(#cfg_attrs)*
                        #[doc(hidden)]
                        #vis(crate) unsafe fn #fn_stub_var(_: u32, _: u32, _: u32, _: u32, _: u32, _: u32) -> u32
                    },
                    Some(EabiAttr {
                        eabi: EabiKind::Arg7,
                        ..
                    }) => quote! {
                        #(#cfg_attrs)*
                        #[doc(hidden)]
                        #vis(crate) unsafe fn #fn_stub_var(_: u32, _: u32, _: u32, _: u32, _: u32, _: u32, _: u32) -> u32
                    },
                    Some(EabiAttr {
                        eabi: EabiKind::Arg8,
                        ..
                    }) => quote! {
                        #(#cfg_attrs)*
                        #[doc(hidden)]
                        #vis(crate) unsafe fn #fn_stub_var(_: u32, _: u32, _: u32, _: u32, _: u32, _: u32, _: u32, _: u32) -> u32
                    },
                    Some(EabiAttr {
                        eabi: EabiKind::I_II_I_RI,
                        ..
                    }) => quote! {
                        #(#cfg_attrs)*
                        #[doc(hidden)]
                        #vis(crate) unsafe fn #fn_stub_var(_: u32, _: u64, _: u32) -> u32
                    },
                    Some(EabiAttr {
                        eabi: EabiKind::I_II_I_RII,
                        ..
                    }) => quote! {
                        #(#cfg_attrs)*
                        #[doc(hidden)]
                        #vis(crate) unsafe fn #fn_stub_var(_: u32, _: u64, _: u32) -> u64
                    },
                    None => {
                        let unsafety = if unsafety.is_some() {
                            // No need as it will show up in the signature already
                            quote! {}
                        } else {
                            quote! {safe}
                        };
                        quote! {
                            #(#attrs)*
                            #[link_name = #fn_libname_link]
                            #vis #unsafety #sig
                        }
                    },
                };

                let register = vec![
                    quote! {"$4"},  // a0
                    quote! {"$5"},  // a1
                    quote! {"$6"},  // a2
                    quote! {"$7"},  // a3
                    quote! {"$8"},  // t0
                    quote! {"$9"},  // t1
                    quote! {"$10"}, // t2
                    quote! {"$11"}, // t3
                ];

                let panic = quote! {
                    #[cfg(not(target_os = "psp"))] {
                        panic!("tried to call PSP system function on non-PSP target");
                    }
                };
                let eabi_fn = match eabi.clone() {
                    Some(EabiAttr {
                        eabi: EabiKind::Arg5,
                        ..
                    }) => {
                        let args: Vec<_> = sig
                            .inputs
                            .iter()
                            .filter_map(|f| match f {
                                syn::FnArg::Receiver(_) => None,
                                syn::FnArg::Typed(pat_type) => match *pat_type.pat.clone() {
                                    syn::Pat::Ident(pat_ident) => Some(pat_ident.ident.clone()),
                                    _ => None,
                                },
                            })
                            .collect();

                        let mut sig = sig.clone();
                        sig.unsafety = None;

                        quote! {
                            #(#attrs)*
                            #[cfg(any(target_os = "psp", doc))]
                            #[allow(non_snake_case, clippy::transmutes_expressible_as_ptr_casts, clippy::missing_safety_doc, clippy::missing_transmute_annotations, clippy::useless_transmute)]
                            #[allow(improper_ctypes, reason = "Rust lint false positive (Rust issue #115457)")]
                            #vis #unsafety extern "C" #sig {
                                #[cfg(target_os = "psp")] {
                                    unsafe {
                                        // ::core::mem::transmute(#crate_path::eabi::i5(#( ::core::mem::transmute( #args ), )* #fn_stub_var))
                                        let result: u32;
                                        ::core::arch::asm!(
                                            ".set noat",
                                            // Call the function pointer
                                            "jalr {func}",
                                            "nop",
                                            #(in(#register) ::core::mem::transmute::<_, u32>(#args),)*
                                            func = in(reg) #fn_stub_var ,
                                            // Result comes back in v0 register ($2)
                                            lateout("$2") result,
                                            // Clobber registers that may be modified by the call
                                            lateout("$8") _, // t0
                                        );

                                        ::core::mem::transmute(result)
                                    }
                                }

                                #panic

                            }
                        }
                    },
                    Some(EabiAttr {
                        eabi: EabiKind::Arg6,
                        ..
                    }) => {
                        let args: Vec<_> = sig
                            .inputs
                            .iter()
                            .filter_map(|f| match f {
                                syn::FnArg::Receiver(_) => None,
                                syn::FnArg::Typed(pat_type) => match *pat_type.pat.clone() {
                                    syn::Pat::Ident(pat_ident) => Some(pat_ident.ident.clone()),
                                    _ => None,
                                },
                            })
                            .collect();

                        let mut sig = sig.clone();
                        sig.unsafety = None;

                        quote! {
                            #(#attrs)*
                            #[cfg(any(target_os = "psp", doc))]
                            #[allow(non_snake_case, clippy::transmutes_expressible_as_ptr_casts, clippy::missing_safety_doc, clippy::missing_transmute_annotations, clippy::useless_transmute)]
                            #[allow(improper_ctypes, reason = "Rust lint false positive (Rust issue #115457)")]
                            #vis #unsafety extern "C" #sig {
                                #[cfg(target_os = "psp")] {
                                    unsafe {
                                        // ::core::mem::transmute(#crate_path::eabi::i5(#( ::core::mem::transmute( #args ), )* #fn_stub_var))
                                        let result: u32;
                                        ::core::arch::asm!(
                                            ".set noat",
                                            // Call the function pointer
                                            "jalr {func}",
                                            "nop",
                                            #(in(#register) ::core::mem::transmute::<_, u32>(#args),)*
                                            func = in(reg) #fn_stub_var ,
                                            // Result comes back in v0 register ($2)
                                            lateout("$2") result,
                                            // Clobber registers that may be modified by the call
                                            lateout("$8") _, // t0
                                            lateout("$9") _, // t1
                                        );

                                        ::core::mem::transmute(result)
                                    }
                                }


                                #panic
                            }

                        }
                    },
                    Some(EabiAttr {
                        eabi: EabiKind::Arg7,
                        ..
                    }) => {
                        let args: Vec<_> = sig
                            .inputs
                            .iter()
                            .filter_map(|f| match f {
                                syn::FnArg::Receiver(_) => None,
                                syn::FnArg::Typed(pat_type) => match *pat_type.pat.clone() {
                                    syn::Pat::Ident(pat_ident) => Some(pat_ident.ident.clone()),
                                    _ => None,
                                },
                            })
                            .collect();

                        let mut sig = sig.clone();
                        sig.unsafety = None;

                        quote! {
                            #(#attrs)*
                            #[cfg(any(target_os = "psp", doc))]
                            #[allow(non_snake_case, clippy::transmutes_expressible_as_ptr_casts, clippy::missing_safety_doc, clippy::missing_transmute_annotations, clippy::useless_transmute)]
                            #[allow(improper_ctypes, reason = "Rust lint false positive (Rust issue #115457)")]
                            #vis #unsafety extern "C" #sig {
                                #[cfg(target_os = "psp")] {
                                    unsafe {
                                        let result: u32;
                                        ::core::arch::asm!(
                                            ".set noat",
                                            // Call the function pointer
                                            "jalr {func}",
                                            "nop",
                                            #(in(#register) ::core::mem::transmute::<_, u32>(#args),)*
                                            func = in(reg) #fn_stub_var ,
                                            // Result comes back in v0 register ($2)
                                            lateout("$2") result,
                                            // Clobber registers that may be modified by the call
                                            lateout("$8") _, // t0
                                            lateout("$9") _, // t1
                                            lateout("$10") _, // t2
                                        );

                                        ::core::mem::transmute(result)
                                    }
                                }

                                #panic
                            }
                        }
                    },
                    Some(EabiAttr {
                        eabi: EabiKind::Arg8,
                        ..
                    }) => {
                        let args: Vec<_> = sig
                            .inputs
                            .iter()
                            .filter_map(|f| match f {
                                syn::FnArg::Receiver(_) => None,
                                syn::FnArg::Typed(pat_type) => match *pat_type.pat.clone() {
                                    syn::Pat::Ident(pat_ident) => Some(pat_ident.ident.clone()),
                                    _ => None,
                                },
                            })
                            .collect();

                        let mut sig = sig.clone();
                        sig.unsafety = None;

                        quote! {
                            #(#attrs)*
                            #[cfg(any(target_os = "psp", doc))]
                            #[allow(non_snake_case, clippy::transmutes_expressible_as_ptr_casts, clippy::missing_safety_doc, clippy::missing_transmute_annotations, clippy::useless_transmute)]
                            #[allow(improper_ctypes, reason = "Rust lint false positive (Rust issue #115457)")]
                            #vis #unsafety extern "C" #sig {
                                #[cfg(target_os = "psp")] {
                                    unsafe {
                                        // ::core::mem::transmute(#crate_path::eabi::i5(#( ::core::mem::transmute( #args ), )* #fn_stub_var))
                                        let result: u32;
                                        ::core::arch::asm!(
                                            ".set noat",
                                            // Call the function pointer
                                            "jalr {func}",
                                            "nop",
                                            #(in(#register) ::core::mem::transmute::<_, u32>(#args),)*
                                            func = in(reg) #fn_stub_var ,
                                            // Result comes back in v0 register ($2)
                                            lateout("$2") result,
                                            // Clobber registers that may be modified by the call
                                            lateout("$8") _, // t0
                                            lateout("$9") _, // t1
                                            lateout("$10") _, // t2
                                            lateout("$11") _, // t3
                                        );

                                        ::core::mem::transmute(result)
                                    }
                                }

                                #panic
                            }
                        }
                    },
                    Some(EabiAttr {
                        eabi: EabiKind::I_II_I_RI,
                        ..
                    }) => {
                        let args: Vec<_> = sig
                            .inputs
                            .iter()
                            .filter_map(|f| match f {
                                syn::FnArg::Receiver(_) => None,
                                syn::FnArg::Typed(pat_type) => match *pat_type.pat.clone() {
                                    syn::Pat::Ident(pat_ident) => Some(pat_ident.ident.clone()),
                                    _ => None,
                                },
                            })
                            .collect();

                        let mut sig = sig.clone();
                        sig.unsafety = None;

                        quote! {
                            #(#attrs)*
                            #[cfg(any(target_os = "psp", doc))]
                            #[allow(non_snake_case, clippy::transmutes_expressible_as_ptr_casts, clippy::missing_safety_doc, clippy::missing_transmute_annotations, clippy::useless_transmute)]
                            #[allow(improper_ctypes, reason = "Rust lint false positive (Rust issue #115457)")]
                            #vis #unsafety extern "C" #sig {
                                #[cfg(target_os = "psp")] {
                                    unsafe {
                                        ::core::mem::transmute(#crate_path::eabi::i_ii_i_ri(#( ::core::mem::transmute( #args ), )* #fn_stub_var))
                                    }
                                }

                                #panic
                            }
                        }
                    },
                    Some(EabiAttr {
                        eabi: EabiKind::I_II_I_RII,
                        ..
                    }) => {
                        let args: Vec<_> = sig
                            .inputs
                            .iter()
                            .filter_map(|f| match f {
                                syn::FnArg::Receiver(_) => None,
                                syn::FnArg::Typed(pat_type) => match *pat_type.pat.clone() {
                                    syn::Pat::Ident(pat_ident) => Some(pat_ident.ident.clone()),
                                    _ => None,
                                },
                            })
                            .collect();

                        let mut sig = sig.clone();
                        sig.unsafety = None;

                        quote! {
                            #(#attrs)*
                            #[cfg(any(target_os = "psp", doc))]
                            #[allow(non_snake_case, clippy::transmutes_expressible_as_ptr_casts, clippy::missing_safety_doc, clippy::missing_transmute_annotations, clippy::useless_transmute)]
                            #[allow(improper_ctypes, reason = "Rust lint false positive (Rust issue #115457)")]
                            #vis #unsafety extern "C" #sig {
                                #[cfg(target_os = "psp")] {
                                    unsafe {
                                        ::core::mem::transmute(#crate_path::eabi::i_ii_i_rii(#( ::core::mem::transmute( #args ), )* #fn_stub_var))
                                    }
                                }


                                #panic
                            }
                        }
                    },
                    None => quote! {},
                };


                fn_stub_code.push(stub_vars);
                fn_stub_extern_items.push(stub_extern);
                fn_eabi_defs.push(eabi_fn);
            }
        }

        let generated = quote! {
            #[cfg(target_os = "psp")]
            #[allow(non_snake_case)]
            mod #mod_name {
                #![allow(non_upper_case_globals, non_snake_case, clippy::missing_safety_doc)]

                #[unsafe(link_section = #resident_section)]
                static #resident_var: [u8; #crate_path::sys::macro_helpers::lib_name_bytes_len(#lib_name)] = #crate_path::sys::macro_helpers::lib_name_bytes(#lib_name);

                #[unsafe(link_section = #nid_start_section)]
                static #nid_start_var: () = ();

                #[unsafe(link_section = #fn_stub_start_section)]
                static #fn_stub_start_var: () = ();

                #[unsafe(link_section = #var_stub_start_section)]
                static #var_stub_start_var: () = ();

                #[unsafe(link_section = #stub_entry_section)]
                static #stub_entry_var: #crate_path::sys::library::StubLibraryEntry = #crate_path::sys::library::StubLibraryEntry {
                    name: #resident_var.as_ptr().cast(),
                    version: (#major_version, #minor_version),
                    flags: #crate_path::sys::LibFlags::from_bits_retain(#flags),
                    len: #crate_path::sys::library::STUB_LIBRARY_ENTRY_TABLE_OLD_LEN,
                    var_stub_count: #var_stub_count,
                    func_stub_count: #fn_stub_count,
                    nid_table: &#nid_start_var as *const () as *const _,
                    func_stub_table: &#fn_stub_start_var as *const () as *const _,
                    var_stub_table: &#var_stub_start_var as *const () as *const _,
                    unk: 0,
                };

                #(#fn_stub_code)*
            }

            #[cfg(any(target_os = "psp", doc))]
            #[allow(non_snake_case)]
            unsafe extern "C" {
                #(#fn_stub_extern_items;)*
            }

            #(#fn_eabi_defs)*
        };

        tokens.extend(generated);
    }
}

pub struct LibInfo {
    lib_name: LitStr,
    flags: LitInt,
    major_version: LitInt,
    minor_version: LitInt,
}

pub struct StubArgs {
    lib_name: Option<LitStr>,
    flags: Option<LitInt>,
    major_version: LitInt,
    minor_version: LitInt,
    use_crate: bool,
}

impl StubArgs {
    pub fn parse(&mut self, meta: ParseNestedMeta) -> syn::Result<()> {
        if meta.path.is_ident("libname") || meta.path.is_ident("name") {
            self.lib_name = meta.value()?.parse()?;
        } else if meta.path.is_ident("flags") {
            self.flags = meta.value()?.parse()?;
        } else if meta.path.is_ident("version") {
            let tuple: ExprTuple = meta.value()?.parse()?;

            if tuple.elems.len() != 2 {
                return Err(Error::new(
                    tuple.span(),
                    "expected a tuple with 2 elements on `version`",
                ));
            }

            let major = tuple.elems.first().expect("expression");
            let minor = tuple.elems.get(1).expect("expression");

            match major {
                Expr::Lit(ExprLit {
                    lit: Lit::Int(val), ..
                }) => self.major_version = val.clone(),
                _ => {
                    return Err(Error::new(
                        major.span(),
                        "expected a integer literal for the major version",
                    ));
                },
            }

            match minor {
                Expr::Lit(ExprLit {
                    lit: Lit::Int(val), ..
                }) => self.minor_version = val.clone(),
                _ => {
                    return Err(Error::new(
                        minor.span(),
                        "expected a integer literal for the major version",
                    ));
                },
            }
        } else if meta.path.is_ident("use_crate") {
            self.use_crate = true;
        } else {
            return Err(Error::new(meta.path.span(), "invalid parameter for `psp_stub`"));
        }

        Ok(())
    }
}

impl Default for StubArgs {
    fn default() -> Self {
        Self {
            lib_name: Default::default(),
            flags: Default::default(),
            major_version: LitInt::new("0", Span::call_site()),
            minor_version: LitInt::new("0", Span::call_site()),
            use_crate: false,
        }
    }
}

struct PspExternItemInfo {
    item: ForeignItem,
    nid: Expr,
    eabi: Option<EabiAttr>,
}

impl PspExternItemInfo {
    fn is_static(&self) -> bool {
        matches!(self.item, ForeignItem::Static(_))
    }

    fn is_func(&self) -> bool {
        matches!(self.item, ForeignItem::Fn(_))
    }
}

impl Parse for PspExternItemInfo {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let item: ForeignItem = input.parse()?;

        match item {
            ForeignItem::Fn(func) => {
                let attrs = func
                    .attrs
                    .iter()
                    .filter(|att| !att.path().is_ident("nid"))
                    .cloned()
                    .collect();

                let nid = func
                    .attrs
                    .iter()
                    .find(|att| att.path().is_ident("nid"))
                    .map(|att| syn::parse2::<NidAttr>(att.meta.to_token_stream()));

                let eabi = func
                    .attrs
                    .iter()
                    .find(|att| att.path().is_ident("eabi"))
                    .map(|att| syn::parse2::<EabiAttr>(att.meta.to_token_stream()));

                let func_name = func.sig.ident.to_string();

                let nid = match nid {
                    Some(nid) => nid?.value,
                    None => {
                        let hash = Sha1::digest(func_name.as_bytes());
                        let mut nid = String::from("0x");
                        for byte in hash.iter().take(4).rev() {
                            nid.push_str(&format!("{byte:02X}"));
                        }
                        Expr::Lit(ExprLit {
                            lit: Lit::Int(LitInt::new(&nid, func.span())),
                            attrs: Vec::new(),
                        })
                    },
                };


                let eabi = match eabi {
                    Some(eabi) => Some(eabi?),
                    None => None,
                };

                let mut item = func.clone();
                item.attrs = attrs;
                let item = ForeignItem::Fn(item);

                Ok(PspExternItemInfo { item, nid, eabi })
            },
            ForeignItem::Static(statik) => {
                let attrs = statik
                    .attrs
                    .iter()
                    .filter(|att| !att.path().is_ident("nid"))
                    .cloned()
                    .collect();

                let nid = statik
                    .attrs
                    .iter()
                    .find(|att| att.path().is_ident("nid"))
                    .map(|att| syn::parse2::<NidAttr>(att.meta.to_token_stream()));

                let statik_name = statik.ident.to_string();

                let nid = match nid {
                    Some(nid) => nid?.value,
                    None => {
                        let hash = Sha1::digest(statik_name.as_bytes());
                        let mut nid = String::from("0x");
                        for byte in hash.iter().take(4).rev() {
                            nid.push_str(&format!("{byte:02X}"));
                        }
                        Expr::Lit(ExprLit {
                            lit: Lit::Int(LitInt::new(&nid, statik.span())),
                            attrs: Vec::new(),
                        })
                    },
                };

                let mut item = statik.clone();
                item.attrs = attrs;
                let item = ForeignItem::Static(item);
                Ok(PspExternItemInfo {
                    item,
                    nid,
                    eabi: None,
                })
            },
            _ => Err(Error::new(
                item.span(),
                "`psp_extern` only supports `static` and `fn` items",
            )),
        }
    }
}

struct NidAttr {
    pub value: Expr,
}

impl Parse for NidAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let meta: Meta = input.parse()?;

        match meta {
            Meta::List(meta_list) => {
                let value: Expr = syn::parse2(meta_list.tokens)?;

                match &value {
                    Expr::Lit(ExprLit { lit, .. }) => match lit {
                        Lit::Int(_) => {},
                        _ => return Err(Error::new(value.span(), "expected a integer literal")),
                    },
                    Expr::If(_) => {},
                    _ => {
                        return Err(Error::new(
                            value.span(),
                            "expected a literal integer or a if expression in constant context",
                        ))
                    },
                }

                Ok(NidAttr { value })
            },
            _ => Err(Error::new(meta.span(), "`nid` attribute syntax is `#[nid({integer})]`")),
        }
    }
}

#[derive(Clone)]
#[allow(unused)]
struct EabiAttr {
    eabi: EabiKind,
    span: Span,
}

impl EabiAttr {
    fn from_meta(input: &Meta) -> syn::Result<Self> {
        match input {
            Meta::List(list) => {
                let ident: Ident = syn::parse2(list.tokens.clone())?;
                if ident == "i5" {
                    Ok(EabiAttr {
                        eabi: EabiKind::Arg5,
                        span: ident.span(),
                    })
                } else if ident == "i6" {
                    Ok(EabiAttr {
                        eabi: EabiKind::Arg6,
                        span: ident.span(),
                    })
                } else if ident == "i7" {
                    Ok(EabiAttr {
                        eabi: EabiKind::Arg7,
                        span: ident.span(),
                    })
                } else if ident == "i8" {
                    Ok(EabiAttr {
                        eabi: EabiKind::Arg8,
                        span: ident.span(),
                    })
                } else if ident == "i_ii_i_ri" {
                    Ok(EabiAttr {
                        eabi: EabiKind::I_II_I_RI,
                        span: ident.span(),
                    })
                } else if ident == "i_ii_i_rii" {
                    Ok(EabiAttr {
                        eabi: EabiKind::I_II_I_RII,
                        span: ident.span(),
                    })
                } else {
                    Err(Error::new(ident.span(), "invalid eabi kind"))
                }
            },
            _ => Err(Error::new(input.span(), "expected something else")),
        }
    }
}

impl Parse for EabiAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let meta = input.parse()?;
        Self::from_meta(&meta)
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
enum EabiKind {
    Arg5,
    Arg6,
    Arg7,
    Arg8,
    I_II_I_RI,
    I_II_I_RII,
}

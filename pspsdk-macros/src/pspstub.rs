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
        } = self;

        let crate_path = if cfg!(feature = "use_crate") {
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

        for fn_stub in items.iter().filter(|e| e.is_func()) {
            let PspExternItemInfo { item, nid } = fn_stub;

            if let ForeignItem::Fn(ForeignItemFn {
                attrs, vis, sig, ..
            }) = item
            {
                let Signature { unsafety, .. } = sig;

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
                    #[unsafe(link_section = #fn_nid_section)]
                    static #fn_nid_var: u32 = #nid;

                    #[unsafe(link_section = #fn_stub_section)]
                    #[unsafe(no_mangle)]
                    static #fn_stub_var: #crate_path::sys::macro_helpers::Stub = #crate_path::sys::macro_helpers::Stub {
                        lib_addr: &#stub_entry_var,
                        nid_addr: &#fn_nid_var
                    };

                };

                let unsafety = if unsafety.is_some() {
                    // No need as it will show up in the signature already
                    quote! {}
                } else {
                    quote! {safe}
                };

                let stub_extern = quote! {
                    #(#attrs)*
                    #[link_name = #fn_libname_link]
                    #vis #unsafety #sig
                };

                fn_stub_code.push(stub_vars);
                fn_stub_extern_items.push(stub_extern);
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
                static #stub_entry_var: #crate_path::sys::library::SceStubLibraryEntry = #crate_path::sys::library::SceStubLibraryEntry {
                    name: #resident_var.as_ptr().cast(),
                    version: (#major_version, #minor_version),
                    flags: #crate_path::sys::SceLibFlags::from_bits_retain(#flags),
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
        }
    }
}

struct PspExternItemInfo {
    item: ForeignItem,
    nid: LitInt,
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

                let func_name = func.sig.ident.to_string();

                let nid = match nid {
                    Some(nid) => nid?.value,
                    None => {
                        let hash = Sha1::digest(func_name.as_bytes());
                        let mut nid = String::from("0x");
                        for byte in hash.iter().take(4).rev() {
                            nid.push_str(&format!("{byte:02X}"));
                        }
                        LitInt::new(&nid, func.span())
                    },
                };

                let mut item = func.clone();
                item.attrs = attrs;
                let item = ForeignItem::Fn(item);

                Ok(PspExternItemInfo { item, nid })
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
                        LitInt::new(&nid, statik.span())
                    },
                };

                let mut item = statik.clone();
                item.attrs = attrs;
                let item = ForeignItem::Static(item);
                Ok(PspExternItemInfo { item, nid })
            },
            _ => Err(Error::new(
                item.span(),
                "`psp_extern` only supports `static` and `fn` items",
            )),
        }
    }
}

struct NidAttr {
    pub value: LitInt,
}

impl Parse for NidAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let meta: Meta = input.parse()?;

        match meta {
            Meta::List(meta_list) => {
                let value: LitInt = syn::parse2(meta_list.tokens)?;

                Ok(NidAttr { value })
            },
            _ => Err(Error::new(meta.span(), "`nid` attribute syntax is `#[nid({integer})]`")),
        }
    }
}

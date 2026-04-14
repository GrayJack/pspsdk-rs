use proc_macro2::Span;
use quote::{quote, ToTokens};
use sha1::{Digest, Sha1};
use syn::{
    parse::Parse, punctuated::Punctuated, Expr, ExprLit, Ident, Lit, LitInt, LitStr, PatLit, Token,
};


pub struct ExportArgs {
    pub libname: LitStr,
    pub major_ver: Expr,
    pub minor_ver: Expr,
    pub flags: Expr,
    pub table: Vec<TableEntry>,
}

impl Parse for ExportArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let libname = input.parse()?;
        let _a: Token![,] = input.parse()?;

        let major_ver: Expr = input.parse()?;
        let _a: Token![,] = input.parse()?;
        let minor_ver: Expr = input.parse()?;
        let _a: Token![,] = input.parse()?;
        let flags: Expr = input.parse()?;
        let _a: Token![,] = input.parse()?;


        let content;
        syn::bracketed!(content in input);
        let table = Punctuated::<TableEntry, Token![,]>::parse_terminated(&content)?
            .into_iter()
            .collect();

        Ok(Self {
            libname,
            major_ver,
            minor_ver,
            flags,
            table,
        })
    }
}

pub struct TableEntry {
    pub kind: EntryKind,
    pub ident: Ident,
    pub nid: Expr, // parsed from => <u32 literal>
}

pub enum EntryKind {
    Function,
    Variable,
}

mod kw {
    syn::custom_keyword!(var);
}

impl Parse for TableEntry {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let kind = if input.peek(kw::var) {
            let _a: kw::var = input.parse()?;
            EntryKind::Variable
        } else if input.peek(Token![static]) {
            let _a: Token![static] = input.parse()?;
            EntryKind::Variable
        } else if input.peek(Token![fn]) {
            let _a: Token![fn] = input.parse()?;
            EntryKind::Function
        } else {
            return Err(syn::Error::new(input.span(), "expected `static` or `fn`"));
        };
        let item: Ident = input.parse()?;

        let nid = if input.peek(Token![:]) {
            let _a: Token![:] = input.parse()?;
            input.parse()?
        } else {
            let hash = Sha1::digest(item.to_string());
            let mut nid = String::from("0x");
            for byte in hash.iter().take(4).rev() {
                nid.push_str(&format!("{byte:02X}"));
            }
            Expr::Lit(ExprLit {
                lit: Lit::Int(LitInt::new(&nid, item.span())),
                attrs: Vec::new(),
            })
        };

        Ok(Self {
            kind,
            ident: item,
            nid,
        })
    }
}

impl ToTokens for TableEntry {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.ident;
        let nid = &self.nid;

        match &self.kind {
            EntryKind::Function => {
                let kw = syn::token::Fn(Span::mixed_site());
                tokens.extend(quote! {#kw #ident : #nid});
            },
            EntryKind::Variable => {
                let kw = syn::token::Static(Span::mixed_site());
                tokens.extend(quote! {#kw #ident : #nid});
            },
        };
    }
}

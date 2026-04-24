use proc_macro2::Span;
use quote::{quote, ToTokens};
use sha1::{Digest, Sha1};
use syn::{
    parse::Parse, punctuated::Punctuated, spanned::Spanned, Expr, ExprLit, ExprPath, Ident, Lit,
    LitInt, LitStr, Token,
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
    pub item: TableEntryItem,
    pub nid: Expr, // parsed from => <u32 literal>
}

pub enum TableEntryItem {
    Ident(Ident),
    Expr(Expr),
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

        let expr: Expr = input.parse()?;

        let item = if let Expr::Path(ExprPath { path, .. }) = expr {
            let ident = path.require_ident()?;
            TableEntryItem::Ident(ident.clone())
        } else {
            TableEntryItem::Expr(expr)
        };

        let nid = if input.peek(Token![:]) {
            let _a: Token![:] = input.parse()?;
            input.parse()?
        } else {
            if let Some(ident) = item.get_ident() {
                let hash = Sha1::digest(ident.to_string());
                let mut nid = String::from("0x");
                for byte in hash.iter().take(4).rev() {
                    nid.push_str(&format!("{byte:02X}"));
                }
                Expr::Lit(ExprLit {
                    lit: Lit::Int(LitInt::new(&nid, item.span())),
                    attrs: Vec::new(),
                })
            } else {
                return Err(syn::Error::new(
                    item.span(),
                    "expression is not a simple identifier, NID specification is required",
                ));
            }
        };

        Ok(Self { kind, item, nid })
    }
}

impl ToTokens for TableEntry {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let item = &self.item;
        let nid = &self.nid;

        match &self.kind {
            EntryKind::Function => {
                let kw = syn::token::Fn(Span::mixed_site());
                tokens.extend(quote! {#kw #item : #nid});
            },
            EntryKind::Variable => {
                let kw = syn::token::Static(Span::mixed_site());
                tokens.extend(quote! {#kw #item : #nid});
            },
        };
    }
}

#[allow(dead_code)]
impl TableEntryItem {
    pub fn is_ident(&self) -> bool {
        matches!(self, Self::Ident(_))
    }

    pub fn is_expr(&self) -> bool {
        matches!(self, Self::Expr(_))
    }

    pub fn get_ident(&self) -> Option<&Ident> {
        match self {
            TableEntryItem::Ident(ident) => Some(ident),
            TableEntryItem::Expr(_) => None,
        }
    }

    pub fn spam(&self) -> Span {
        match self {
            TableEntryItem::Ident(ident) => ident.span(),
            TableEntryItem::Expr(expr) => expr.span(),
        }
    }
}

impl ToTokens for TableEntryItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            TableEntryItem::Ident(ident) => tokens.extend(ident.to_token_stream()),
            TableEntryItem::Expr(expr) => tokens.extend(expr.to_token_stream()),
        }
    }
}

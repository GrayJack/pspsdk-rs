use syn::{parse::Parse, punctuated::Punctuated, Token};

use crate::ExportArgs;


pub struct ExportsArgs {
    pub exports: Vec<ExportArgs>,
}

impl Parse for ExportsArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let entries = Punctuated::<ExportArgs, Token![;]>::parse_terminated(input)?
            .into_iter()
            .collect();

        Ok(Self { exports: entries })
    }
}

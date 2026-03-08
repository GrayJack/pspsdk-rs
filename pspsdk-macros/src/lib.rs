use proc_macro::TokenStream;
use quote::ToTokens;
use syn::Result;

use crate::pspstub::{PspStub, StubArgs};

mod pspstub;

#[proc_macro_attribute]
pub fn psp_stub(attr: TokenStream, item: TokenStream) -> TokenStream {
    match psp_stub_impl(attr, item) {
        Ok(ts) => ts,
        Err(err) => err.into_compile_error().into(),
    }
}

fn psp_stub_impl(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let args: StubArgs = syn::parse(attr)?;

    let psp_stub = PspStub::parse(args, item)?;

    Ok(psp_stub.to_token_stream().into())
}

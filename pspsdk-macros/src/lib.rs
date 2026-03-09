use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_macro_input, Result};

use crate::pspstub::{PspStub, StubArgs};

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

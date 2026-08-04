use syn::{parse::Parse, spanned::Spanned, Expr, RangeLimits};


pub enum PspFwCfgArg {
    Lit(u32),
    Range(u32, u32, RangeLimits),
}

impl Parse for PspFwCfgArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let item: Expr = input.parse()?;

        match item {
            Expr::Lit(syn::ExprLit { lit, .. }) => match lit {
                syn::Lit::Int(lit_int) => {
                    let int = lit_int.base10_parse();
                    match int {
                        Ok(int) if !(30..=661).contains(&int) => Err(syn::Error::new(
                            lit_int.span(),
                            "expected value in the range of `30` to `661`",
                        )),
                        Ok(int) => Ok(PspFwCfgArg::Lit(int)),
                        Err(err) => Err(err),
                    }
                },
                lit => Err(syn::Error::new(lit.span(), "expected decimal integer literal")),
            },
            Expr::Range(syn::ExprRange {
                start, end, limits, ..
            }) => {
                let mut start_val = 100;
                let mut end_val = 661;
                if let Some(start_exp) = start {
                    match start_exp.as_ref() {
                        Expr::Lit(syn::ExprLit { lit, .. }) => match lit {
                            syn::Lit::Int(lit_int) => {
                                let int = lit_int.base10_parse();
                                match int {
                                    Ok(int) if !(30..=661).contains(&int) => {
                                        return Err(syn::Error::new(
                                            lit_int.span(),
                                            "expected value in the range of `30` to `661`",
                                        ))
                                    },
                                    Ok(int) => start_val = int,
                                    Err(err) => return Err(err),
                                }
                            },
                            lit => {
                                return Err(syn::Error::new(
                                    lit.span(),
                                    "expected decimal integer literal",
                                ))
                            },
                        },
                        expr => {
                            return Err(syn::Error::new(expr.span(), "expected integer literal"))
                        },
                    }
                }

                if let Some(end_exp) = end {
                    match end_exp.as_ref() {
                        Expr::Lit(syn::ExprLit { lit, .. }) => match lit {
                            syn::Lit::Int(lit_int) => {
                                let int = lit_int.base10_parse();
                                match int {
                                    Ok(int) if !(30..=661).contains(&int) => {
                                        return Err(syn::Error::new(
                                            lit_int.span(),
                                            "expected value in the range of `30` to `661`",
                                        ))
                                    },
                                    Ok(int) => end_val = int,
                                    Err(err) => return Err(err),
                                }
                            },
                            lit => {
                                return Err(syn::Error::new(
                                    lit.span(),
                                    "expected decimal integer literal",
                                ))
                            },
                        },
                        expr => {
                            return Err(syn::Error::new(expr.span(), "expected integer literal"))
                        },
                    }
                } else {
                    // half open with None on end, means we go to end
                    if let RangeLimits::HalfOpen(_) = &limits {
                        end_val = 662;
                    }
                }

                Ok(PspFwCfgArg::Range(start_val, end_val, limits))
            },
            expr => Err(syn::Error::new(expr.span(), "expected integer literal or integer range")),
        }
    }
}

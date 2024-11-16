use std::path::PathBuf;

use proc_macro2::TokenStream;
use quote::quote;
use sabry_intrnl::scoper::ArbitraryScope;
use syn::{
    parse::{Parse, ParseStream},
    Ident,
};

use super::{ArbitraryStyleBlock, ArbitraryStyleSyntax};

/// Syntax:
/// `#ident(:$syntax)? { $code }`
///
/// `$syntax`: sass/scss
///
/// `$code`: intended to be valid based on $syntax
pub fn scssy_macro_impl(input: TokenStream, source_path: Option<PathBuf>) -> TokenStream {
    let MacroSyntax {
        ident,
        syntax,
        code,
    } = match syn::parse::Parser::parse2(
        |input: ParseStream<'_>| MacroSyntax::parse_syn(input, source_path),
        input,
    ) {
        Ok(m) => m,
        Err(e) => return e.to_compile_error(),
    };

    // quick raffia syntax check
    match ArbitraryScope::from_source(syntax.0, ident.clone(), &code.code) {
        Ok(_) => {}
        Err(e) => return syn::Error::new(code.span, format!("{e:?}")).into_compile_error(),
    }

    let sourcesass = code.to_string();
    let macro_doc = format!("Arbitrary {:?} code declared with `scssy!`. Pretty usable in tandem with `usey!` and `buildy` at build time", syntax);

    quote! {
        #[doc = #macro_doc]
        #[macro_export]
        macro_rules! #ident {
            () => {#sourcesass};
            (syntax) => {#syntax};
        }
    }
}

pub struct MacroSyntax {
    ident: Ident,
    syntax: ArbitraryStyleSyntax,
    code: ArbitraryStyleBlock,
}

impl MacroSyntax {
    pub fn parse_syn(
        input: syn::parse::ParseStream,
        source_path: Option<PathBuf>,
    ) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        let syntax = input.parse::<ArbitraryStyleSyntax>()?;

        let code = match source_path {
            Some(sp) => ArbitraryStyleBlock::parse_syn(input, Some(sp))?,
            None => input.parse::<ArbitraryStyleBlock>()?,
        };

        Ok(Self {
            ident,
            syntax,
            code,
        })
    }
}

impl Parse for MacroSyntax {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Self::parse_syn(input, None)
    }
}

#[cfg(test)]
mod test {}

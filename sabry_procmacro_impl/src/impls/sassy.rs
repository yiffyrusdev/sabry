use proc_macro2::TokenStream;
use quote::quote;
use sabry_intrnl::scoper::ArbitraryScope;
use syn::{parse::Parse, Ident};

use super::{ArbitraryStyleBlock, ArbitraryStyleSyntax};

/// Syntax:
/// `#ident(:$syntax)? { $code }`
///
/// `$syntax`: sass/scss
///
/// `$code`: intended to be valid based on $syntax
pub fn sassy_macro_impl(input: TokenStream) -> TokenStream {
    let MacroSyntax {
        ident,
        syntax: mode,
        code,
    } = match syn::parse2::<MacroSyntax>(input) {
        Ok(m) => m,
        Err(e) => return e.to_compile_error(),
    };

    // quick raffia syntax check
    match ArbitraryScope::from_source(mode.0, ident.clone(), &code.code) {
        Ok(_) => {}
        Err(e) => return syn::Error::new(code.span, format!("{e:?}")).into_compile_error(),
    }

    let sourcesass = code.to_string();
    let macro_doc = format!("Arbitrary {:?} code declared with `sassy!`. Pretty usable in tandem with `usey!` and `magic()` at build time", mode);

    quote! {
        #[doc = #macro_doc]
        #[macro_export]
        macro_rules! #ident {
            () => {#sourcesass};
            (syntax) => {#mode};
        }
    }
}

pub struct MacroSyntax {
    ident: Ident,
    syntax: ArbitraryStyleSyntax,
    code: ArbitraryStyleBlock,
}

impl Parse for MacroSyntax {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        let mode = input.parse::<ArbitraryStyleSyntax>()?;
        let code = input.parse::<ArbitraryStyleBlock>()?;

        Ok(Self {
            ident,
            syntax: mode,
            code,
        })
    }
}

#[cfg(test)]
mod test {}

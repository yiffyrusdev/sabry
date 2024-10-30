use proc_macro2::TokenStream;
use quote::quote;
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

    let sourcesass = code.to_string();
    let identname = ident.to_string();
    let macro_doc = format!("Arbitrary {:?} code declared with `sassy!`. Pretty usable in tandem with `usey!` and `magic()` at build time", mode);

    quote! {
        #[doc = #macro_doc]
        #[macro_export]
        macro_rules! #ident {
            () => {#sourcesass};
            (module) => {#identname};
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
        #[cfg(feature = "sass")]
        use regex::Regex;

        let ident = input.parse::<Ident>()?;
        let mode = input.parse::<ArbitraryStyleSyntax>()?;
        let mut code = input.parse::<ArbitraryStyleBlock>()?;

        // this is because of how parsing works. Need to find more elegant way though
        // TODO: find a more elegant way
        code.code = Regex::new("\n    ").expect("BUG: failed to build regex for indentation SASS").replace_all(&code.code, "\n").to_string();

        Ok(Self {
            ident,
            syntax: mode,
            code,
        })
    }
}

#[cfg(test)]
mod test {

}

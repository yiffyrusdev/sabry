use std::{fmt::Debug, fs, path::PathBuf, str::FromStr};

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, TokenStreamExt, quote};
use regex::Regex;
use sabry_intrnl::syntax::ostrta::OneSyntaxToRuleThemAll;
use syn::{Ident, LitStr, Token, braced, parse::Parse};

pub mod scssy;
pub mod styly;
pub mod usey;

/// Macro input for arbitrary style code
///
/// Also supports string literal token instead of braces,
/// in which mode will look for relative file
///
/// Does not work nice with rust-analyzer though
#[derive(Clone, Debug)]
pub struct ArbitraryStyleBlock {
    code: String,
    span: Span,
}

impl ArbitraryStyleBlock {
    pub fn span(&self) -> Span {
        self.span
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn parse_syn(input: syn::parse::ParseStream, path_prefix: PathBuf) -> syn::Result<Self> {
        let (code, span) = if let Ok(path_tok) = input.parse::<LitStr>() {
            let path = {
                let p = path_tok.value();
                if let Some(pp) = p.strip_prefix("./") {
                    path_prefix.join(pp)
                } else {
                    PathBuf::from_str(&p).unwrap() // infallible
                }
            };

            let fullpath = match path.canonicalize() {
                Ok(cp) => cp,
                Err(e) => {
                    return Err(syn::Error::new(
                        path_tok.span(),
                        format!(
                            "Could not use path {path:?}: {:?}. If the path is relative and 'nightly' feature flag is set - this is likely false-positive",
                            e.kind()
                        ),
                    ));
                }
            };

            let iofile = match fs::read(&fullpath) {
                Ok(bf) => bf,
                Err(_) => {
                    return Err(syn::Error::new(
                        path_tok.span(),
                        format!("Could not read file at {fullpath:?}"),
                    ));
                }
            };
            let code = String::from_utf8_lossy(&iofile).to_string();

            (code, path_tok.span())
        } else {
            let s;
            braced!(s in input);
            if let Ok(stream) = s.parse::<LitStr>() {
                let c = stream.value();
                // shift the entire code for the first line ident
                let ident_regex = Regex::new(r"\n\s{4}").expect("BUG: base ident regex at sabry_procmacro_impl/src/impls/mod.rs:: impl Parse for ArbitraryStyleBlock");
                let c = ident_regex.replace_all(&c, "\n").to_string();
                (c, stream.span())
            } else {
                return Err(syn::Error::new(
                    s.span(),
                    "Use \"\" within the braces to specify your SASS/SCSS. Unquoted style syntax is reserved for the future. Unquoted SCSS/SASS doesnt make sense though, as you won't benefit from it in rust file.\n\ntip: use `{\"style\"}` instead of `{style}`",
                ));
            }
        };

        Ok(Self { code, span })
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for ArbitraryStyleBlock {
    fn to_string(&self) -> String {
        self.code.clone()
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct ArbitraryStyleSyntax(OneSyntaxToRuleThemAll);

impl From<ArbitraryStyleSyntax> for OneSyntaxToRuleThemAll {
    fn from(value: ArbitraryStyleSyntax) -> Self {
        value.0
    }
}

impl ToTokens for ArbitraryStyleSyntax {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.0 {
            OneSyntaxToRuleThemAll::Sass => tokens.append_all(quote! {"sass"}),
            OneSyntaxToRuleThemAll::Scss => tokens.append_all(quote! {"scss"}),
        }
    }
}

impl TryFrom<&str> for ArbitraryStyleSyntax {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self(OneSyntaxToRuleThemAll::try_from(value)?))
    }
}

impl Parse for ArbitraryStyleSyntax {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        match input.parse::<Token![:]>() {
            Ok(_) => { /* Syntax is set */ }
            Err(_) => {
                /* Syntax is not set, falling to default */
                return Ok(Self::default());
            }
        }
        let ident = input.parse::<Ident>()?;

        match Self::try_from(ident.to_string().as_str()) {
            Ok(v) => Ok(v),
            Err(_) => Err(syn::Error::new(
                ident.span(),
                format!(
                    "Available syntax are `sass` and `scss`, omit to use default {:?} syntax",
                    Self::default()
                ),
            )),
        }
    }
}

use std::{fmt::Debug, fs, path::PathBuf, str::FromStr};

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use regex::Regex;
use sabry_intrnl::compiler::CompilerSyntax;
use syn::{braced, parse::Parse, Ident, LitStr, Token};

pub mod sassy;
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
}

impl Parse for ArbitraryStyleBlock {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let (code, span) = if let Ok(path_tok) = input.parse::<LitStr>() {
            let path = path_tok.value();

            let path = match PathBuf::from_str(&path) {
                Ok(p) => p,
                Err(_) => {
                    return Err(syn::Error::new(
                        path_tok.span(),
                        format!("Could not read {} as a path: Infallible", path),
                    ))
                }
            };
            let fullpath = match path.canonicalize() {
                Ok(cp) => cp,
                Err(e) => {
                    return Err(syn::Error::new(
                        path_tok.span(),
                        format!("Could not use path {path:?}: {:?}", e.kind()),
                    ))
                }
            };

            let iofile = match fs::read(&fullpath) {
                Ok(bf) => bf,
                Err(_) => {
                    return Err(syn::Error::new(
                        path_tok.span(),
                        format!("Could not read file at {fullpath:?}"),
                    ))
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
                return Err(syn::Error::new(s.span(), "Use \"\" within the braces to specify your SASS/SCSS. Unquoted style syntax is reserved for the future. Unquoted SCSS/SASS doesnt make sense though, as you won't benefit from it in rust file.\n\ntip: use `{\"style\"}` instead of `{style}`"));
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

#[derive(Clone, Copy)]
pub enum ArbitraryStyleSyntax {
    Scss,
    Sass,
}

impl From<ArbitraryStyleSyntax> for CompilerSyntax {
    fn from(value: ArbitraryStyleSyntax) -> Self {
        match value {
            ArbitraryStyleSyntax::Sass => Self::Sass,
            ArbitraryStyleSyntax::Scss => Self::Scss,
        }
    }
}

impl ToTokens for ArbitraryStyleSyntax {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Sass => tokens.append_all(quote! {"sass"}),
            Self::Scss => tokens.append_all(quote! {"scss"}),
        }
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

        match ident.to_string().as_str() {
            "sass" => Ok(Self::Sass),
            "scss" => Ok(Self::Scss),
            _ => Err(syn::Error::new(
                ident.span(),
                format!(
                    "Available syntax are `sass` and `scss`, omit to use default {:?} syntax",
                    Self::default()
                ),
            )),
        }
    }
}

impl Debug for ArbitraryStyleSyntax {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Sass => "sass",
            Self::Scss => "scss",
        };
        write!(f, "{name}")
    }
}

impl Default for ArbitraryStyleSyntax {
    fn default() -> Self {
        Self::Scss
    }
}

impl From<ArbitraryStyleSyntax> for raffia::Syntax {
    fn from(value: ArbitraryStyleSyntax) -> Self {
        match value {
            ArbitraryStyleSyntax::Sass => Self::Sass,
            ArbitraryStyleSyntax::Scss => Self::Scss,
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn arbitrary_style_block_sass() {
        use super::ArbitraryStyleBlock;

        let code = "
    #a
        co: red
    .sel
        co: white
    &-dark
        & > div
            co: blue";
        let expect_code = "
#a
    co: red
.sel
    co: white
&-dark
    & > div
        co: blue";
        let input = format!("{{\"{code}\"}}");
        let block = syn::parse_str::<ArbitraryStyleBlock>(&input).unwrap();

        assert_eq!(expect_code, block.code);
    }

    #[test]
    fn arbitrary_style_block_scss() {
        use super::ArbitraryStyleBlock;

        let code = "
    #a {
        c: r;
    }
    .b.c {
        c: 'into';
    }";
        let expect_code = "
#a {
    c: r;
}
.b.c {
    c: 'into';
}";
        let input = format!("{{\"{code}\"}}");
        let block = syn::parse_str::<ArbitraryStyleBlock>(&input).unwrap();

        assert_eq!(expect_code, block.code)
    }
}

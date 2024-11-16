use std::path::PathBuf;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use sabry_intrnl::{
    compiler::CompilerAdapter,
    config::SabryConfig,
    scoper::{apply_basic_rusty_member_gen_rules, ArbitraryScope, ScopedSelector},
};
use syn::{
    parse::{Parse, ParseStream},
    Ident, Token,
};

use super::{ArbitraryStyleBlock, ArbitraryStyleSyntax};

/// Syntax:
/// `pub? #ident(:$syntax)? { $code } \#?`
///
/// `$syntax`: sass/scss
///
/// `$code`: intended to be valid based on $syntax
///
/// `pub` modifier makes scope public
///
/// `const` modifier completely changes the behaviour of styling, and
/// makes CSS exposed as constant
///
/// `#` shebang generates machine-readable and well-parsible but invalid rust code for building purposes
/// instead of scope module.
///
/// The machine-readable output may also be forced by use `machine_readable: true` arg on `parse_macro_syntax` function
/// without modifying tokenstream
pub fn styly_macro_impl(input: TokenStream, source_path: Option<PathBuf>) -> TokenStream {
    let config = match SabryConfig::require() {
        Ok(c) => c,
        Err(e) => {
            return syn::Error::new(
                Span::call_site(),
                format!(
                    "Could not read sabry configuration required by this macro: {:?}",
                    e
                ),
            )
            .to_compile_error()
        }
    };
    let ms = match parse_macro_syntax(input, source_path) {
        Ok(ms) => ms,
        Err(e) => return e.to_compile_error(),
    };

    let scope = match ArbitraryScope::from_source(ms.syntax.0, ms.scope.clone(), &ms.code.code) {
        Ok(s) => match s.hashed(&config.hash) {
            Ok(hs) => Ok(hs),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    };
    let scope = match scope {
        Ok(s) => s,
        Err(e) => {
            return syn::Error::new(
                Span::call_site(),
                format!("Could not generate scope: {:?}", e),
            )
            .to_compile_error()
        }
    };

    match ms.generator {
        ScopeGenerator::Module { public, constant } => {
            let scope_hash = scope.hash.as_str();
            let scope_ident = scope.original_scope.name.clone();
            let scope_wrapper_ident =
                syn::parse_str::<Ident>(&scope_ident.to_string().to_uppercase())
                    .expect("BUG: We just converted valid Ident to string and uppercased");
            let scope_vis = if public {
                quote! {pub}
            } else {
                quote! {}
            };

            let nests = scope.original_scope.adapter().nesting_selectors();
            let special_nesting_members = nests
                .iter()
                .filter_map(|ns| {
                    ns.suffix.clone().and_then(|s| {
                        let interp = s
                            .as_sass_interpolated()
                            .and_then(|i| i.elements.first())
                            .and_then(|e| e.as_static().map(|s| s.raw.to_string()));

                        interp
                    })
                })
                .map(|s| {
                    let ident = apply_basic_rusty_member_gen_rules(&s);
                    let fnident = syn::parse_str::<Ident>(format!("_{ident}").as_str())
                        .expect("BUG: invalid ident for nesting selector formed");
                    let formatstr = format!("{{c}}{}", &s);
                    let doc = format!("Nesting selector. Originated from '&{s}'");

                    quote! {
                        #[doc = #doc]
                        pub fn #fnident(c: &str) -> String {format!(#formatstr)}
                    }
                });

            let scope_members = scope
                .hashed_selectors
                .iter()
                .filter_map(|hs| {
                    hs.html_ident.clone().map(|html| {
                        let name = match hs.sel {
                            ScopedSelector::Class(_) => "class",
                            ScopedSelector::Id(_) => "id",
                            ScopedSelector::Tag(_) => "tagname",
                        };

                        (name, hs.sel.gen_rusty_ident(), html, hs.css_ident.clone())
                    })
                })
                .map(|(name, ident, html, css)| {
                    let docs = format!("'{}' {}. CSS selector '{}'", ident, name, css);
                    quote! {
                        #[doc = #docs]
                        #[allow(non_upper_case_globals)]
                        pub const #ident : &str = #html ;
                    }
                });

            let mod_docs = format!(
                "'{}' style scope. The wrapper class for scoped tagnames is {}",
                &scope_ident, &scope_wrapper_ident
            );
            let wrp_docs = format!("wrapper class for '{}' scope. If you have any tagname selectors - they should live as children of element with this class applied.", &scope_ident);

            if constant {
                let compiler = CompilerAdapter::new(config.clone());
                let css = match compiler.compile_module(ms.syntax.0, &scope.hashed_code) {
                    Ok(c) => c,
                    Err(e) => {
                        return syn::Error::new(
                            ms.code.span,
                            format!("Could not compile const inline: {e:?}"),
                        )
                        .to_compile_error()
                    }
                };

                let const_docs = format!("The compiled CSS style for {} scope", &scope_ident);
                let const_wrapper_ident = syn::parse_str::<Ident>(
                    format!("{}_CSS", &scope_ident.to_string().to_uppercase()).as_str(),
                )
                .expect("BUG: We just converted valid Ident to string and uppercased");

                quote! {
                    #[doc = #const_docs]
                    #scope_vis const #const_wrapper_ident: &str = #css ;
                    #[doc = #wrp_docs]
                    #scope_vis const #scope_wrapper_ident : &str = #scope_hash ;
                    #[doc = #mod_docs]
                    #scope_vis mod #scope_ident {
                        #(#scope_members)*
                        #(#special_nesting_members)*
                    }
                }
            } else {
                quote! {
                    #[doc = #wrp_docs]
                    #scope_vis const #scope_wrapper_ident : &str = #scope_hash ;
                    #[doc = #mod_docs]
                    #scope_vis mod #scope_ident {
                        #(#scope_members)*
                        #(#special_nesting_members)*
                    }
                }
            }
        }
        ScopeGenerator::MachineReadable => {
            quote! {}
        }
    }
}

// reusable function, that does not return tokenstream for machine-processing at build time
pub fn parse_macro_syntax(
    input: TokenStream,
    source_path: Option<PathBuf>,
) -> Result<MacroSyntax, syn::Error> {
    syn::parse::Parser::parse2(
        |input: ParseStream<'_>| MacroSyntax::parse_syn(input, source_path),
        input,
    )
}

#[derive(Debug)]
pub enum ScopeGenerator {
    Module { public: bool, constant: bool },
    MachineReadable,
}

impl Parse for ScopeGenerator {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let public = input.parse::<Token![pub]>().is_ok();
        let constant = input.parse::<Token![const]>().is_ok();

        // would be nice to implement compile-time CSS compilation to const
        // with `const` modifier, so I'll leave it this way for now
        let this = Self::Module { public, constant };

        Ok(this)
    }
}

#[derive(Debug)]
pub struct MacroSyntax {
    /// The way macro should be expanded
    pub generator: ScopeGenerator,
    /// Scope identifier
    pub scope: Ident,
    /// Used stytax
    pub syntax: ArbitraryStyleSyntax,
    /// Style code, either from rust or read from file
    pub code: ArbitraryStyleBlock,
}

impl MacroSyntax {
    pub fn parse_syn(
        input: syn::parse::ParseStream,
        source_path: Option<PathBuf>,
    ) -> syn::Result<Self> {
        let mut generator = input.parse::<ScopeGenerator>()?;
        let scope = input.parse::<Ident>()?;
        let syntax = input.parse::<ArbitraryStyleSyntax>()?;
        let code = if source_path.is_some() {
            ArbitraryStyleBlock::parse_syn(input, source_path)
        } else {
            input.parse::<ArbitraryStyleBlock>()
        }?;

        if input.parse::<Token![#]>().is_ok() {
            generator = ScopeGenerator::MachineReadable
        }

        Ok(Self {
            generator,
            scope,
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

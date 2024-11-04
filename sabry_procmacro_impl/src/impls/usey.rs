use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Parse, punctuated::Punctuated, Macro, Token};

/// Syntax:
/// `#(pub? #macro,)*`
pub fn usey_macro_impl(input: TokenStream) -> TokenStream {
    let MacroSyntax { stylecalls } = match syn::parse2(input) {
        Ok(ms) => ms,
        Err(e) => return e.to_compile_error(),
    };

    let stylecall_pairs = stylecalls
        .iter()
        .map(|c| c.to_contract_pair("__sabry_pub_module"));

    quote! {
        vec![
            #(#stylecall_pairs,)*
        ]
    }
}

#[derive(Clone)]
pub struct StyleCall {
    vis_pub: bool,
    call: Macro,
}

impl StyleCall {
    pub fn to_contract_pair(&self, pub_module: &str) -> TokenStream {
        let call_code = &self.call;
        let call_path = &self.call.path;
        let call_bang = &self.call.bang_token;
        // the following calls are expected by macro genereted by `scssy!`
        let call_syntax = quote! {#call_path #call_bang (syntax)};

        let module_name = call_path
            .segments
            .last()
            .map(|l| l.ident.to_string())
            .expect("BUG: failed to get identifier for macro call");

        if self.vis_pub {
            quote! {
                (format!("{}.{}", #pub_module, #call_syntax), #call_code .to_string())
            }
        } else {
            quote! {
                (format!("{}.{}", #module_name, #call_syntax), #call_code .to_string())
            }
        }
    }
}

impl Parse for StyleCall {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis_pub = input.parse::<Token![pub]>().is_ok();
        let call = input.parse::<Macro>()?;

        Ok(Self { vis_pub, call })
    }
}

pub struct MacroSyntax {
    stylecalls: Vec<StyleCall>,
}

impl Parse for MacroSyntax {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let calls = Punctuated::<StyleCall, Token![,]>::parse_separated_nonempty(input)?;

        let stylecalls = calls.iter().cloned().collect();
        Ok(Self { stylecalls })
    }
}

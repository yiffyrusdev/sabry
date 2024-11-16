use std::{
    fs, io,
    path::{Path, PathBuf},
};

use proc_macro2::TokenStream;
use sabry_procmacro_impl::impls::styly;
use syn::{spanned::Spanned, visit::Visit};

#[derive(Debug, thiserror::Error)]
pub enum FileVisitError {
    #[error("tt read file")]
    Read(#[from] io::Error),
    #[error("styly! macro found, yet invalid")]
    Styly(#[from] syn::Error),
}

/// Visit file, returning either [StylyVisitor] or error
pub fn visit_file(path: &Path) -> Result<StylyVisitor, FileVisitError> {
    let content = fs::read_to_string(path)?;
    let code_file = syn::parse_file(&content)?;

    let mut styly_visitor = StylyVisitor {
        found_stylys: vec![],
        path: path.to_owned(),
    };
    styly_visitor.visit_file(&code_file);

    Ok(styly_visitor)
}

/// Syn Visitor, which will look at every `styly!` macro,
/// parse its input tokenstream and avaluate it, capturing [styly::MacroSyntax] result
///
/// ## Panics
///
/// If either macro body is not a [TokenStream], or [styly::parse_macro_syntax] is not [Ok]
#[derive(Debug)]
pub struct StylyVisitor {
    pub found_stylys: Vec<styly::MacroSyntax>,
    pub path: PathBuf,
}

impl<'ast> Visit<'ast> for StylyVisitor {
    fn visit_item_macro(&mut self, node: &'ast syn::ItemMacro) {
        if node.mac.path.get_ident().is_some_and(|i| i == "styly") {
            let body = node.mac.parse_body::<TokenStream>().unwrap_or_else(|e| {
                panic!(
                    "could not parse `styly!` macro at {:?}: {e:?}",
                    node.mac.span()
                )
            });
            let macro_data =
                styly::parse_macro_syntax(body, self.path.parent().map(|p| p.to_owned()))
                    .unwrap_or_else(|e| {
                        panic!(
                            "could not parse `styly!` macro at {:?}: {e:?}",
                            node.mac.span()
                        )
                    });
            self.found_stylys.push(macro_data);
        }
    }
}

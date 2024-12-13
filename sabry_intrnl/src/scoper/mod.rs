use std::fmt::Debug;

use cfg_if::cfg_if;
use hash::ScopeHash;
use raffia::{Span, Spanned};
use regex::Regex;

use crate::{
    config::SabryHashConfig,
    syntax::{ostrta::OneSyntaxToRuleThemAll, StylesheetAdapter},
};

pub mod hash;

#[derive(thiserror::Error)]
pub enum ScopeError {
    #[error("Raffia reports parse error")]
    Raffia(raffia::error::ErrorKind, String),
}

impl Debug for ScopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let a = match self {
            Self::Raffia(kind, source) => format!("{kind}: at {source}"),
        };

        write!(f, "{a}")
    }
}

/// Struct which represents arbitrary style scope with known style syntax and name identifier
pub struct ArbitraryScope<'s> {
    /// Ident name for the current scope
    pub name: syn::Ident,
    adapter: StylesheetAdapter<'s>,
}

impl<'s> ArbitraryScope<'s> {
    /// Parse given source code with given syntax and assign given name
    pub fn from_source(
        syntax: OneSyntaxToRuleThemAll,
        name: syn::Ident,
        source: &'s str,
    ) -> Result<Self, ScopeError> {
        let adapter = match StylesheetAdapter::new(source, syntax) {
            Ok(a) => a,
            Err(e) => {
                let source = &source[e.span.start..e.span.end];
                return Err(ScopeError::Raffia(e.kind, source.to_string()));
            }
        };

        Ok(Self { adapter, name })
    }

    /// Consume arbitrary scope and create a [HashedScope] in its basis
    ///
    /// This function calls [HashedScope::new] under the hood
    pub fn hashed(self, config: &SabryHashConfig) -> Result<HashedScope<'s>, ScopeError> {
        let hash = ScopeHash::new(&self, config);
        HashedScope::new(hash, self)
    }

    /// Borrow the underlying [StylesheetAdapter]
    pub fn adapter(&self) -> &StylesheetAdapter<'s> {
        &self.adapter
    }
}

/// Hashed scope which holds hashed style code,
/// vector of hashed selectors and the calculated hash itself.
///
/// Construction with [HashedScope::new] consumes [ArbitraryScope] and does all the job
pub struct HashedScope<'s> {
    /// Original arbitrary scope
    pub original_scope: ArbitraryScope<'s>,
    /// Hashed style code
    pub hashed_code: String,
    /// Collection of hashed selectors
    pub hashed_selectors: Vec<HashedSelector>,

    /// Given hash to the scope
    pub hash: ScopeHash,
}

impl<'s> HashedScope<'s> {
    /// Consume [ArbitraryScope],
    /// calculate the hash,
    /// hash all the supported selectors
    /// and construct hashed source code
    ///
    /// Heavy operation.
    pub fn new(hash: ScopeHash, scope: ArbitraryScope<'s>) -> Result<Self, ScopeError> {
        let origin_code = scope.adapter().source();
        let mut hashed_code = String::with_capacity(scope.adapter().source().len());

        // get classes
        let classes = scope.adapter().class_selectors();
        let classes = classes.iter().map(|c| HashedSelector::from_class(&hash, c));

        // get ids
        let ids = scope.adapter().id_selectors();
        let ids = ids.iter().map(|c| HashedSelector::from_id(&hash, c));

        // get tags
        let tags = scope.adapter().type_selectors();
        let tags = tags
            .iter()
            .filter_map(|t| t.as_tag_name())
            .map(|c| HashedSelector::from_tag(&hash, c));

        // get global selectors
        let globs = scope.adapter().glob_modified_selectors();
        let globs = globs
            .iter()
            .filter_map(|g| {
                // we do ignore the latter selectors from :global(a, b) because there's no clear decision:
                // how should we treat :glob(a, b) c {}
                // as `a c {}` `b c {}`
                // or as `a, b c {}` ?
                g.arg.clone().and_then(|a| {
                    if let raffia::ast::PseudoClassSelectorArgKind::SelectorList(list) = a.kind {
                        list.selectors
                            .first()
                            .cloned()
                            .map(|fcl| (g.span.clone(), fcl.span.clone()))
                    } else {
                        None
                    }
                })
            })
            .map(|c| HashedSelector::from_glob_mod(&hash, c, scope.adapter().source()));

        let mut hashed_selectors = classes
            .chain(ids)
            .chain(tags)
            .chain(globs)
            .collect::<Vec<_>>();

        // sorting by span start is important because of how hashed code construction works
        hashed_selectors.sort_by(|a, b| {
            let astart = a.sel.as_arbitrary().span.start;
            let bstart = b.sel.as_arbitrary().span.start;

            astart.cmp(&bstart)
        });

        let mut last_term_span: usize = 0;
        for sel in &hashed_selectors {
            let span = sel.sel.as_arbitrary().span.clone();
            hashed_code.push_str(&origin_code[last_term_span..span.start]);
            hashed_code.push_str(&sel.css_ident);
            last_term_span = span.end;
        }
        hashed_code.push_str(&origin_code[last_term_span..]);

        Ok(Self {
            original_scope: scope,
            hashed_code,
            hashed_selectors,
            hash,
        })
    }
}

/// Struct which represents hashed selector inside of [HashedScope].
///
/// Does not hold information about hash itself, as meant to be inside of scope.
#[derive(PartialEq, Eq, Hash)]
pub struct HashedSelector {
    /// Basic unrelated to hash info about this selector
    pub sel: ScopedSelector,
    /// CSS-ish identifier for this selector, which matches what's in the hashed style code
    pub css_ident: String,
    /// HTML-ish identifier for this selector, which will match [Self::css_ident] in browser
    pub html_ident: Option<String>,
}

impl HashedSelector {
    pub fn from_glob_mod(hash: &ScopeHash, sel: (Span, Span), source: &str) -> Self {
        let sel = ScopedSelector::from_glob_mod(sel, source);
        let css_ident = Self::make_hashed_css(&sel, hash);
        let html_ident = Self::make_hashed_html(&sel, hash);

        Self {
            sel,
            css_ident,
            html_ident,
        }
    }

    /// Construct the [HashedSelector], with given hash and parsed class selector
    pub fn from_class(hash: &ScopeHash, sel: &raffia::ast::ClassSelector) -> Self {
        let sel = ScopedSelector::from_class(sel);
        let css_ident = Self::make_hashed_css(&sel, hash);
        let html_ident = Self::make_hashed_html(&sel, hash);

        Self {
            sel,
            css_ident,
            html_ident,
        }
    }

    /// Construct the [HashedSelector], with given hash and parsed id selector
    pub fn from_id(hash: &ScopeHash, sel: &raffia::ast::IdSelector) -> Self {
        let sel = ScopedSelector::from_id(sel);
        let css_ident = Self::make_hashed_css(&sel, hash);
        let html_ident = Self::make_hashed_html(&sel, hash);

        Self {
            sel,
            css_ident,
            html_ident,
        }
    }

    /// Construct the [HashedSelector], with given hash and parsed tagname selector
    pub fn from_tag(hash: &ScopeHash, sel: &raffia::ast::TagNameSelector) -> Self {
        let sel = ScopedSelector::from_tag(sel);
        let css_ident = Self::make_hashed_css(&sel, hash);
        let html_ident = Self::make_hashed_html(&sel, hash);

        Self {
            sel,
            css_ident,
            html_ident,
        }
    }

    /// Transform this selector into hashed version for CSS-ish language
    ///
    /// Cooperates with the [HashedSelector::make_hashed_html], so
    /// whats this function returns will be usable in HTML with to_hashed_html_def()
    pub fn make_hashed_css(value: &ScopedSelector, hash: &ScopeHash) -> String {
        match value {
            // Class scoping is done with class composition
            ScopedSelector::Class(a) => {
                format!("{}.{}", hash.as_str(), a.ident)
            }
            // As we dont want to use two HTML props for a single ID,
            // ID scoping is done with id modification
            ScopedSelector::Id(a) => {
                cfg_if! {
                    if #[cfg(feature = "lepty-scoping")] {
                        format!("{}.{}", a.ident, hash.as_str())
                    } else {
                        format!("{}-{}", hash.as_str(), a.ident)
                    }
                }
            }
            ScopedSelector::Tag(a) => {
                cfg_if! {
                    if #[cfg(feature = "lepty-scoping")] {
                        format!("{}.{}", a.ident, hash.as_str())
                    } else {
                        format!(".{} {}", hash.as_str(), a.ident)
                    }
                }
            }
            ScopedSelector::Glob { raw, .. } => raw.clone(),
        }
    }

    /// Transform this selector into hashed version valid for HTML-ish insertion
    ///
    /// Cooperates with the [HashedSelector::make_hashed_css], so
    /// whats this function returns will be usable in CSS with to_hashed_code()
    ///
    /// Not every hashed selector is presentable for HTML-ish use: like `div`, in that case
    /// returns [None]
    pub fn make_hashed_html(value: &ScopedSelector, _hash: &ScopeHash) -> Option<String> {
        match value {
            // Class scoping is done with class composition
            ScopedSelector::Class(a) => {
                cfg_if! {
                    if #[cfg(feature = "lepty-scoping")] {
                        Some(a.ident.to_string())
                    } else {
                        Some(format!("{} {}", _hash.as_str(), a.ident))
                    }
                }
            }
            // As we dont want to use two HTML props for a single ID,
            // ID scoping is done with id modification
            ScopedSelector::Id(a) => {
                cfg_if! {
                    if #[cfg(feature = "lepty-scoping")] {
                        Some(a.ident.to_string())
                    } else {
                        Some(format!("{}-{}", _hash.as_str(), a.ident))
                    }
                }
            }
            ScopedSelector::Tag(_) => None,
            ScopedSelector::Glob { .. } => None,
        }
    }
}

/// Just any unary CSS-ish selector
#[derive(PartialEq, Eq, Hash)]
pub struct ArbitrarySelector {
    /// CSS-ish identifier
    pub ident: String,
    /// Span in original CSS code where it comes from
    pub span: Span,
}

/// Any unary CSS-ish selector, that's able to be scoped, and then hashed
#[derive(PartialEq, Eq, Hash)]
pub enum ScopedSelector {
    Class(ArbitrarySelector),
    Id(ArbitrarySelector),
    Tag(ArbitrarySelector),
    Glob {
        origin: ArbitrarySelector,
        inner_span: Span,
        raw: String,
    },
}

impl ScopedSelector {
    /// Borrow the underlying [ArbitrarySelector]
    pub fn as_arbitrary(&self) -> &ArbitrarySelector {
        match self {
            Self::Class(a) => a,
            Self::Id(a) => a,
            Self::Tag(a) => a,
            Self::Glob { origin, .. } => origin,
        }
    }

    /// Generate rusty member ident based on selector type and CSS ident
    pub fn gen_rusty_ident(&self) -> Option<syn::Ident> {
        let arb = &self.as_arbitrary().ident;
        let basic = apply_basic_rusty_member_gen_rules(arb);

        let ready = match self {
            Self::Class(_) => Some(basic),
            Self::Id(_) => Some(format!("the{basic}")),
            Self::Tag(_) => Some(format!("any{basic}")),
            Self::Glob { .. } => None,
        };

        ready.map(|rs| {
            syn::parse_str::<syn::Ident>(&rs).expect("BUG: formed ident is not Rust Ident")
        })
    }

    /// Construct this from glob complex selector
    pub fn from_glob_mod(s: (Span, Span), source: &str) -> Self {
        let inner_span = s.1;
        let origin_ident = source[s.0.start..s.0.end].to_owned();

        Self::Glob {
            origin: ArbitrarySelector {
                span: s.0,
                ident: origin_ident,
            },
            raw: source[inner_span.start..inner_span.end].to_owned(),
            inner_span,
        }
    }

    /// Construct this from class selector
    pub fn from_class(s: &raffia::ast::ClassSelector) -> Self {
        let lit = s
            .name
            .as_literal()
            .expect("BUG: class selector is not a literal");
        Self::Class(ArbitrarySelector {
            ident: lit.raw.to_string(),
            span: lit.span().clone(),
        })
    }

    /// Construct this from id selector
    pub fn from_id(s: &raffia::ast::IdSelector) -> Self {
        let lit = s
            .name
            .as_literal()
            .expect("BUG: id selector is not a literal");
        Self::Id(ArbitrarySelector {
            ident: lit.raw.to_string(),
            span: lit.span().clone(),
        })
    }

    /// Construct this from tagname selector
    pub fn from_tag(s: &raffia::ast::TagNameSelector) -> Self {
        let lit = s
            .name
            .name
            .as_literal()
            .expect("BUG: tag selector is not a literal");
        Self::Tag(ArbitrarySelector {
            ident: lit.raw.to_string(),
            span: lit.span().clone(),
        })
    }
}

pub fn apply_basic_rusty_member_gen_rules(source: &str) -> String {
    let omit_regex = Regex::new(r"(^\-)|(\-$)|[^a-zA-Z0-9\-\_]")
        .expect("BUG: can not build omition regex for rusty member generation");

    // omit forbidden chars
    let cleaned = omit_regex.replace_all(source, "").to_string();

    // other not-so-obvious rules
    let mut target = String::with_capacity(cleaned.len());
    let mut next_uppercase: bool = false;
    for (i, c) in cleaned.chars().enumerate() {
        // first gidit is prepended with 'n'
        if i == 0 && c.is_numeric() {
            target.push('n');
            target.push(c);
            continue;
        }
        // dash is omitted and next is uppercase if may be
        if c == '-' {
            next_uppercase = true;
            continue;
        }
        target.push(if next_uppercase {
            next_uppercase = false;
            c.to_ascii_uppercase()
        } else {
            c
        });
    }
    target
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use cfg_if::cfg_if;
    use syn::Ident;

    use crate::{
        config::SabryHashConfig,
        scoper::{hash::ScopeHash, HashedScope},
        syntax::ostrta::OneSyntaxToRuleThemAll,
    };

    use super::ArbitraryScope;

    #[test]
    fn scope_hash_codegen() {
        let code =
            ".cls1{color:red; &-dark{color: black;} #id1 {color:green;} div {color:blue;}} .cls3#id2{color: black;}";
        let hash = ScopeHash::test_init("F2kf8nMs".into());

        cfg_if! {
            if #[cfg(feature = "lepty-scoping")]{
                let expect_code = ".F2kf8nMs.cls1{color:red; &-dark{color: black;} #id1.F2kf8nMs {color:green;} div.F2kf8nMs {color:blue;}} .F2kf8nMs.cls3#id2.F2kf8nMs{color: black;}";
            } else {
                let expect_code = ".F2kf8nMs.cls1{color:red; &-dark{color: black;} #F2kf8nMs-id1 {color:green;} .F2kf8nMs div {color:blue;}} .F2kf8nMs.cls3#F2kf8nMs-id2{color: black;}";
            }
        }

        cfg_if! {
            if #[cfg(feature = "lepty-scoping")] {
                let expect_selector_htmls = HashSet::from([
                    "cls1".to_string(),
                    "id1".to_string(),
                    "cls3".to_string(),
                    "id2".to_string(),
                    "".to_string(), // comes from `div` which has no html hashed code, still want to check presense
                ]);
            } else {
                let expect_selector_htmls = HashSet::from([
                    "F2kf8nMs cls1".to_string(),
                    "F2kf8nMs-id1".to_string(),
                    "F2kf8nMs cls3".to_string(),
                    "F2kf8nMs-id2".to_string(),
                    "".to_string(), // comes from `div` which has no html hashed code, still want to check presense
                ]);
            }
        }

        let scope = ArbitraryScope::from_source(
            OneSyntaxToRuleThemAll::Scss,
            syn::parse_str("scope1").unwrap(),
            code,
        )
        .unwrap();
        let scope = HashedScope::new(hash, scope).unwrap();
        let scope_selector_htmls = scope
            .hashed_selectors
            .iter()
            .map(|hs| hs.html_ident.clone().unwrap_or_default())
            .collect::<HashSet<_>>();

        // check that original is untouched
        assert_eq!(code, scope.original_scope.adapter().source());
        // check that hashed matches what we expect
        assert_eq!(expect_code, scope.hashed_code);
        // check that HTML selector hashed code matches what we expect
        assert_eq!(expect_selector_htmls, scope_selector_htmls);
    }

    #[test]
    fn rusty_idents() {
        let code = "
.cls-1{
    color:red;
    &-dark{
        color: black;
    }
    #id-1 {
        color:green;
    }
    .-txt-of {
        color:blue;
    }
}
.-c_ls3#4id{
    color: black;
}";
        let expect_ident_names = HashSet::from([
            "cls1".to_string(),
            "theid1".to_string(),
            "c_ls3".to_string(),
            "then4id".to_string(),
            "txtOf".to_string(),
        ]);

        let hash_cfg = SabryHashConfig::default();
        let scope = ArbitraryScope::from_source(
            OneSyntaxToRuleThemAll::Scss,
            syn::parse_str::<Ident>("scope2oiej").unwrap(),
            code,
        )
        .unwrap()
        .hashed(&hash_cfg)
        .unwrap();

        let sels = scope
            .hashed_selectors
            .iter()
            .map(|s| s.sel.gen_rusty_ident())
            .flat_map(|i| i.map(|is| is.to_string()))
            .collect::<HashSet<_>>();

        assert_eq!(expect_ident_names, sels);
    }
}

#![doc = include_str!("../README.md")]
#![cfg_attr(feature = "nightly", feature(proc_macro_span))]

use cfg_if::cfg_if;
use proc_macro::TokenStream;
use sabry_procmacro_impl::impls::{
    scssy::scssy_macro_impl, styly::styly_macro_impl, usey::usey_macro_impl,
};

/// Macro that makes sass code rusty.
///
/// Accepts arbitrary (yet valid) sass(wip)/scss code and
/// makes it available as rust macro
///
/// ## Usage
///
/// ```ignore
/// scssy!(scssmodule {
///     /* SCSS */
/// });
/// scssy!(sassmodule:sass {
///     /* SASS (WIP) */
/// });
/// scssy!(scssmod2:scss {
///     /* SCSS */
/// });
/// ```
/// Or read code from file
/// ```ignore
/// scssy!(filemodule "src/styles/a.scss");
/// scssy!(filemodule:sass "src/styles/b.sass");
/// ```
///
/// \[SCSS syntax requires 'scss' feature\]
/// \[SASS syntax requires 'sass' feature (WIP)\]
///
#[proc_macro]
pub fn scssy(input: TokenStream) -> TokenStream {
    cfg_if! {
        if #[cfg(feature = "nightly")] {
            use proc_macro::Span;
            let source_path = Span::call_site().source_file().path().parent().map(|p| p.to_owned());
        } else {
            let source_path = None;
        }
    }
    scssy_macro_impl(input.into(), source_path).into()
}

/// Macro that brings sass code into app
///
/// Accepts arbitrary (yet valid) sass(wip)/scss code
/// and creates public/private module with scoped(hashed)
/// selector members.
///
/// Feel free to use it with qualified path, but dont alias it with `use sabry::styly as whetever`.
///
/// ## Usage
///
/// Write style in rusty code (rust-analyzer wont autocomplete your scope members)
///
/// ```ignore
/// use sabry::styly;
///
/// styly!(scope1 {
///     /* SCSS */
/// });
/// styly!(scope2:sass {
///     /* SASS (WIP) */
/// });
/// styly!(scope3:scss {
///     /* SCSS */
/// });
/// styly!(pub scope4 {
///     /* SCSS which is available as <mod>::scope4 */
/// });
/// styly!(const scope5 {
///     /* SCSS which wount be bundled, instead SCOPE5_STYLE constant is introduced with CSS */
/// });
/// styly!(pub const scope6 {
///     /* SCSS, which is like 'const scope5' but publically available like 'pub scope4'*/
/// });
/// ```
/// Or use SCSS file and get syntax highlighting and autocompletion:
/// ```ignore
/// styly!(penguin "src/components/penguin.scss");
/// styly!(pub page "src/routes/page.scss");
/// styly!(pub const embed:sass "src/embedded.sass");
/// ```
///
/// \[SCSS syntax requires 'scss' feature\]
/// \[SASS syntax requires 'sass' feature (WIP)\]
///
/// ### About `const` scope
///
/// Const-flavored styly macro will generate SCOPENAME_CSS constant available in runtime, which contains
/// compiled CSS. Note, that only possibility to `@use` something - is to have sabry as build time, so that const styly
/// macro could read all modules.
///
/// ### Not meant to be used, yet still available
///
/// ```ignore
/// styly!(notascope {
///     /* produces machine-readable invalid rust code */
/// }#);
/// ```
#[proc_macro]
pub fn styly(input: TokenStream) -> TokenStream {
    cfg_if! {
        if #[cfg(feature = "nightly")] {
            use proc_macro::Span;
            let source_path = Span::call_site().source_file().path().parent().map(|p| p.to_owned());
        } else {
            let source_path = None;
        }
    }

    styly_macro_impl(input.into(), source_path).into()
}

/// Macro that brings external sass code into app
///
/// Intended to be used in tandem with `sabry::buildy` function in `build.rs` file
///
/// ## Usage
///
/// ```ignore
/// use sharedstyles::tokens;
/// use brandstyle::tokens as brand_tokens;
///
/// usey!(
///     theme!(),
///     tokens!()
/// );
/// ```
///
#[proc_macro]
pub fn usey(input: TokenStream) -> TokenStream {
    usey_macro_impl(input.into()).into()
}

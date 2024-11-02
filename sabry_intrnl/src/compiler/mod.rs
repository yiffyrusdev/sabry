use std::fmt::Debug;

use lightningcss::{
    error::{MinifyErrorKind, PrinterErrorKind},
    printer::PrinterOptions,
    stylesheet::MinifyOptions,
    targets::Targets,
};

use crate::{config::SabryConfig, syntax::ostrta::OneSyntaxToRuleThemAll};

/// Convenience wrapper on Grass and Lightningcss
pub struct CompilerAdapter {
    config: SabryConfig,
}

impl CompilerAdapter {
    pub fn new(config: SabryConfig) -> Self {
        Self { config }
    }

    /// Compile given SASS/SCSS into CSS
    /// with respect to self.config
    pub fn compile_module(
        &self,
        syntax: OneSyntaxToRuleThemAll,
        code: &str,
    ) -> Result<String, SabryCompilerError> {
        let options = grass::Options::from(&self.config).input_syntax(syntax.into());

        let mut css = grass::from_string(code, &options)?;

        css = match self.lightningcss(&css) {
            Ok(c) => c,
            Err(e) => return Err(e),
        };

        Ok(css)
    }

    /// Perform lightningcss transformations on given css
    /// with respect to self.config.css.minify on minification
    pub fn lightningcss(&self, css: &str) -> Result<String, SabryCompilerError> {
        let mut lightsheet = match lightningcss::stylesheet::StyleSheet::parse(
            css,
            lightningcss::stylesheet::ParserOptions::default(),
        ) {
            Ok(s) => s,
            Err(e) => {
                return Err(SabryCompilerError::LightParse {
                    kind: e.kind.to_string(),
                    loc: e.loc,
                })
            }
        };

        let targets = Targets {
            browsers: Some(self.config.lightningcss.targets.clone().into()),
            ..Default::default()
        };

        let printer_options = PrinterOptions {
            minify: self.config.css.minify,
            targets,
            ..Default::default()
        };

        let minify_options = MinifyOptions {
            targets,
            ..Default::default()
        };

        if self.config.css.minify {
            lightsheet.minify(minify_options)?
        }

        let css = match lightsheet.to_css(printer_options) {
            Ok(c) => {
                drop(lightsheet);
                c.code
            }
            Err(e) => return Err(SabryCompilerError::LightPrint(e)),
        };

        Ok(css)
    }
}

impl From<&SabryConfig> for grass::Options<'_> {
    fn from(config: &SabryConfig) -> Self {
        let mut options = grass::Options::default();
        options = options.style(if config.css.minify {
            grass::OutputStyle::Compressed
        } else {
            grass::OutputStyle::Expanded
        });
        options = options.load_path(&config.sass.intermediate_dir);

        options
    }
}

#[derive(thiserror::Error)]
pub enum SabryCompilerError {
    #[error("Could not compile into CSS with grass")]
    GrassCompile(#[from] Box<grass::Error>),
    #[error("Could not print compiled CSS with lightningcss")]
    LightPrint(#[from] lightningcss::error::Error<PrinterErrorKind>),
    #[error("Could not minify CSS with lightningcss")]
    LightMinify(#[from] lightningcss::error::Error<MinifyErrorKind>),
    #[error("Could not parse CSS with lightningcss")]
    LightParse {
        kind: String,
        loc: Option<lightningcss::error::ErrorLocation>,
    },
}

impl Debug for SabryCompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let explain = match self {
            Self::LightPrint(e) => format!(
                "{} at {:?}",
                e.kind,
                e.loc
                    .clone()
                    .map(|l| format!("file {}, line {}, col {}", l.filename, l.line, l.column))
            ),
            Self::GrassCompile(err) => format!("{err}"),
            Self::LightMinify(e) => format!(
                "{} at {:?}",
                e.kind,
                e.loc
                    .clone()
                    .map(|l| format!("file {}, line {}, col {}", l.filename, l.line, l.column))
            ),
            Self::LightParse { kind, loc } => format!(
                "{kind} at {:?}",
                loc.clone()
                    .map(|l| format!("file {}, line {}, col {}", l.filename, l.line, l.column))
            ),
        };

        write!(f, "{explain}")
    }
}

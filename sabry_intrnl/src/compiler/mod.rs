use std::fmt::Debug;

use lightningcss::{
    error::{MinifyErrorKind, PrinterErrorKind},
    printer::PrinterOptions,
    stylesheet::MinifyOptions,
    targets::Targets,
};

use crate::config::SabryConfig;

/// Convenience wrapper on Grass and Lightningcss
pub struct CompilerAdapter {
    config: SabryConfig,
}

impl CompilerAdapter {
    pub fn new(config: SabryConfig) -> Self {
        Self { config }
    }

    pub fn compile_module(
        &self,
        syntax: CompilerSyntax,
        code: &str,
    ) -> Result<String, SabryCompilerError> {
        let options = grass::Options::from(&self.config).input_syntax(syntax.into());

        let mut css = grass::from_string(code, &options)?;

        if self.config.css.minify {
            //its lightningcss time!
            let mut lightsheet = match lightningcss::stylesheet::StyleSheet::parse(
                &css,
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

            lightsheet.minify(minify_options)?;

            css = match lightsheet.to_css(printer_options) {
                Ok(c) => {
                    drop(lightsheet);
                    c.code
                }
                Err(e) => return Err(SabryCompilerError::LightPrint(e)),
            };
        }

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

/// Unification wrapper over all the Syntax enums
#[derive(Clone, Copy)]
pub enum CompilerSyntax {
    Sass,
    Scss,
}

impl From<raffia::Syntax> for CompilerSyntax {
    fn from(value: raffia::Syntax) -> Self {
        match value {
            raffia::Syntax::Sass => Self::Sass,
            raffia::Syntax::Scss => Self::Scss,
            // TODO: fallbacky error-prone
            _ => Self::Scss,
        }
    }
}

impl From<CompilerSyntax> for grass::InputSyntax {
    fn from(value: CompilerSyntax) -> Self {
        match value {
            CompilerSyntax::Scss => Self::Scss,
            CompilerSyntax::Sass => Self::Sass,
        }
    }
}

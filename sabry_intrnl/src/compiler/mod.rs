use lightningcss::{printer::PrinterOptions, stylesheet::MinifyOptions, targets::Targets};

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
                Err(_) => return Err(SabryCompilerError::Lightningcss()),
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

            if lightsheet.minify(minify_options).is_err() {
                return Err(SabryCompilerError::Lightningcss());
            }

            css = match lightsheet.to_css(printer_options) {
                Ok(c) => {
                    drop(lightsheet);
                    c.code
                }
                Err(_) => return Err(SabryCompilerError::Lightningcss()),
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

#[derive(Debug, thiserror::Error)]
pub enum SabryCompilerError {
    #[error("Could not compile into CSS with grass")]
    GrassCompile(#[from] Box<grass::Error>),
    #[error("Could not parse compiled CSS with lightningcss")]
    Lightningcss(),
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

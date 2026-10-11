use crate::{conf::SabryConfig, style::syntax::SyntaxKind};

/// Convenience wrapper on Grass and Lightningcss
pub struct CompilerAdapter {
    #[allow(dead_code)] // without 'compile' feature, still want to set this
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
        syntax: SyntaxKind,
        code: &str,
    ) -> Result<String, SabryCompilerError> {
        let importer = sasso::FsImporter::new(self.config.sass.modules.clone().unwrap_or_default());
        let options = sasso::Options::new()
            .with_style(if self.config.css.minify {
                sasso::OutputStyle::Compressed
            } else {
                sasso::OutputStyle::Expanded
            })
            .with_importer(&importer)
            .with_syntax(syntax.into());

        Ok(sasso::compile(code, &options)?)
    }
}

#[derive(thiserror::Error)]
pub enum SabryCompilerError {
    #[error("Could not compile into CSS with sasso")]
    Compile(#[from] sasso::Error),
}

impl std::fmt::Debug for SabryCompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let explain = match self {
            Self::Compile(err) => format!("{err}"),
        };

        write!(f, "{explain}")
    }
}

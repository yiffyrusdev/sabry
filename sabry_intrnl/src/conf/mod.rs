use std::{path::PathBuf, str::FromStr};

#[cfg(feature = "config-manifest")]
pub mod manifest;

/// Sabry configuration, as it is from package.metadata.sabry
#[derive(Default, Clone)]
#[cfg_attr(feature = "config-manifest", derive(serde::Deserialize))]
#[cfg_attr(feature = "config-manifest", serde(default))]
pub struct SabryConfig {
    pub css: SabryCssConfig,
    pub sass: SabrySassConfig,
    pub hash: SabryHashConfig,
}

impl SabryConfig {
    /// Require sabry configuration for the current project from every place in the code
    #[cfg(feature = "config-manifest")]
    pub fn require() -> Result<Self, manifest::ManifestError> {
        let manifest = manifest::read_manifest()?;
        Ok(Self::from(&manifest))
    }
}

#[cfg(feature = "config-manifest")]
impl From<&manifest::ValuableManifest> for SabryConfig {
    fn from(value: &manifest::ValuableManifest) -> Self {
        value
            .package
            .as_ref()
            .and_then(|p| p.metadata.as_ref())
            .and_then(|m| m.sabry.as_ref())
            .map_or(Self::default(), |sbr| sbr.clone())
    }
}

/// Sabry `css` configuration table
#[derive(Clone)]
#[cfg_attr(feature = "config-manifest", derive(serde::Deserialize))]
#[cfg_attr(feature = "config-manifest", serde(default))]
pub struct SabryCssConfig {
    pub bundle: Option<PathBuf>,
    pub scopes: Option<PathBuf>,
    pub prelude: Vec<PathBuf>,
    #[cfg_attr(
        feature = "config-manifest",
        serde(default = "SabryCssConfig::default_minify")
    )]
    pub minify: bool,
}

impl SabryCssConfig {
    fn default_minify() -> bool {
        true
    }
}

impl Default for SabryCssConfig {
    fn default() -> Self {
        Self {
            bundle: None,
            scopes: None,
            prelude: vec![],
            minify: Self::default_minify(),
        }
    }
}

/// Sabry `sass` configuration table
///
/// (SASS/SCSS related config)
#[derive(Clone)]
#[cfg_attr(feature = "config-manifest", derive(serde::Deserialize))]
#[cfg_attr(feature = "config-manifest", serde(default))]
pub struct SabrySassConfig {
    #[cfg_attr(
        feature = "config-manifest",
        serde(default = "SabrySassConfig::default_scanroot")
    )]
    pub scanroot: PathBuf,
    pub modules: Option<Vec<PathBuf>>,
    pub prelude: Option<Vec<PathBuf>>,
    #[cfg_attr(
        feature = "config-manifest",
        serde(default = "SabrySassConfig::default_intermediate_dir")
    )]
    pub intermediate_dir: PathBuf,
    #[cfg_attr(
        feature = "config-manifest",
        serde(default = "SabrySassConfig::default_module_name_collision")
    )]
    pub module_name_collision: BehavSassModCollision,
}

impl SabrySassConfig {
    fn default_scanroot() -> PathBuf {
        PathBuf::from_str("src")
            .expect("Infallible: default SabrySassConfig::scanroot into PathBuf")
    }
    fn default_intermediate_dir() -> PathBuf {
        PathBuf::from_str("target/.sabry/sass")
            .expect("Infallibe: default SabrySassConfig::intermediate_dir into PathBuf")
    }
    fn default_module_name_collision() -> BehavSassModCollision {
        BehavSassModCollision::default()
    }
}

impl Default for SabrySassConfig {
    fn default() -> Self {
        Self {
            scanroot: Self::default_scanroot(),
            modules: None,
            prelude: None,
            intermediate_dir: Self::default_intermediate_dir(),
            module_name_collision: Self::default_module_name_collision(),
        }
    }
}

/// Sabry `hash` configuration table
#[derive(Clone)]
#[cfg_attr(feature = "config-manifest", derive(serde::Deserialize))]
#[cfg_attr(feature = "config-manifest", serde(default))]
pub struct SabryHashConfig {
    #[cfg_attr(
        feature = "config-manifest",
        serde(default = "SabryHashConfig::default_size")
    )]
    pub size: usize,
    #[cfg_attr(
        feature = "config-manifest",
        serde(default = "SabryHashConfig::default_collision")
    )]
    pub collision: BehavHashCollision,
    #[cfg_attr(
        feature = "config-manifest",
        serde(default = "SabryHashConfig::default_use_scope_name")
    )]
    pub use_scope_name: bool,
    #[cfg_attr(
        feature = "config-manifest",
        serde(default = "SabryHashConfig::default_use_item_names")
    )]
    pub use_item_names: bool,
    #[cfg_attr(
        feature = "config-manifest",
        serde(default = "SabryHashConfig::default_use_code_size")
    )]
    pub use_code_size: bool,
    #[cfg_attr(
        feature = "config-manifest",
        serde(default = "SabryHashConfig::default_use_code_text")
    )]
    pub use_code_text: bool,
}

impl SabryHashConfig {
    fn default_size() -> usize {
        6
    }
    fn default_collision() -> BehavHashCollision {
        BehavHashCollision::default()
    }
    fn default_use_scope_name() -> bool {
        true
    }
    fn default_use_item_names() -> bool {
        false
    }
    fn default_use_code_size() -> bool {
        true
    }
    fn default_use_code_text() -> bool {
        false
    }
}

impl Default for SabryHashConfig {
    fn default() -> Self {
        Self {
            size: Self::default_size(),
            collision: Self::default_collision(),
            use_scope_name: Self::default_use_scope_name(),
            use_code_size: Self::default_use_code_size(),
            use_code_text: Self::default_use_code_text(),
            use_item_names: Self::default_use_item_names(),
        }
    }
}

/// \[package.metadata.sabry.sass\].module_name_collision option
#[derive(Default, Clone, Copy)]
#[cfg_attr(feature = "config-manifest", derive(serde::Deserialize))]
#[cfg_attr(feature = "config-manifest", serde(rename_all = "lowercase"))]
pub enum BehavSassModCollision {
    #[default]
    Merge,
    Error,
}

/// \[package.metadata.sabry.hash\].collision option
#[derive(Default, Clone, Copy)]
#[cfg_attr(feature = "config-manifest", derive(serde::Deserialize))]
#[cfg_attr(feature = "config-manifest", serde(rename_all = "lowercase"))]
pub enum BehavHashCollision {
    #[default]
    Ignore,
    Error,
}

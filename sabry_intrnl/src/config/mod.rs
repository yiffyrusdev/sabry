pub mod manifest;
use manifest::{ManifestError, ValuableManifest};
use serde::Deserialize;

/// Sabry configuration, as it is from package.metadata.sabry
#[derive(Default, Deserialize, Clone)]
pub struct SabryConfig {
    #[serde(default = "SabryCssConfig::default")]
    pub css: SabryCssConfig,
    #[serde(default = "SabrySassConfig::default")]
    pub sass: SabrySassConfig,
    #[serde(default = "SabryHashConfig::default")]
    pub hash: SabryHashConfig,
    #[serde(default = "SabryGrassConfig::default")]
    pub grass: SabryGrassConfig,
    #[serde(default = "SabryLightCssConfig::default")]
    pub lightningcss: SabryLightCssConfig,
}

impl SabryConfig {
    /// Require sabry configuration for the current project from every place in the code
    pub fn require() -> Result<Self, ManifestError> {
        let manifest = manifest::read_manifest()?;
        Ok(Self::from(&manifest))
    }
}

impl From<&ValuableManifest> for SabryConfig {
    fn from(value: &ValuableManifest) -> Self {
        value
            .package
            .as_ref()
            .and_then(|p| p.metadata.as_ref())
            .and_then(|m| m.sabry.as_ref())
            .map_or(Self::default(), |sbr| sbr.clone())
    }
}

/// Sabry `css` configuration table
#[derive(Deserialize, Clone)]
pub struct SabryCssConfig {
    pub bundle: Option<String>,
    pub scopes: Option<String>,
    pub bundle_prelude: Option<Vec<String>>,
    #[serde(default = "SabryCssConfig::default_minify")]
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
            bundle_prelude: None,
            minify: Self::default_minify(),
        }
    }
}

/// Sabry `sass` configuration table
///
/// (SASS/SCSS related config)
#[derive(Deserialize, Clone)]
pub struct SabrySassConfig {
    #[serde(default = "SabrySassConfig::default_scanroot")]
    pub scanroot: String,
    pub modules: Option<Vec<String>>,
    #[serde(default = "SabrySassConfig::default_intermediate_dir")]
    pub intermediate_dir: String,
    #[serde(default = "SabrySassConfig::default_module_name_collision")]
    pub module_name_collision: BehavSassModCollision,
}

impl SabrySassConfig {
    fn default_scanroot() -> String {
        "src".into()
    }
    fn default_intermediate_dir() -> String {
        "target/.sabry/sass".into()
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
            intermediate_dir: Self::default_intermediate_dir(),
            module_name_collision: Self::default_module_name_collision(),
        }
    }
}

/// Sabry `hash` configuration table
#[derive(Deserialize, Clone)]
pub struct SabryHashConfig {
    #[serde(default = "SabryHashConfig::default_size")]
    pub size: usize,
    #[serde(default = "SabryHashConfig::default_collision")]
    pub collision: BehavHashCollision,
    #[serde(default = "SabryHashConfig::default_use_scope_name")]
    pub use_scope_name: bool,
    #[serde(default = "SabryHashConfig::default_use_item_names")]
    pub use_item_names: bool,
    #[serde(default = "SabryHashConfig::default_use_code_size")]
    pub use_code_size: bool,
    #[serde(default = "SabryHashConfig::default_use_code_text")]
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

/// [grass] specific configuration for sabry
#[derive(Default, Deserialize, Clone)]
pub struct SabryGrassConfig {}

/// [lightningcss] specific configuration for sabry
#[derive(Default, Deserialize, Clone)]
pub struct SabryLightCssConfig {
    pub targets: SabryLightTargets,
}

/// \[package.metadata.sabry.lightningcss.targets\] configuration.
///
/// This is a convenience layer under [lightningcss::targets::Browsers] structure accepted
/// by lightningcss.
///
/// This structure is a part of [SabryConfig], which faces developer with more convenient
/// `safari = "13.2"` syntax.
#[derive(Default, Deserialize, Clone)]
pub struct SabryLightTargets {
    pub android: Option<String>,
    pub chrome: Option<String>,
    pub edge: Option<String>,
    pub firefox: Option<String>,
    pub ie: Option<String>,
    pub ios_saf: Option<String>,
    pub opera: Option<String>,
    pub safari: Option<String>,
    pub samsung: Option<String>,
}

impl SabryLightTargets {
    pub fn parse_ver(ver: &str) -> u32 {
        let mut vercode: u32 = 0;
        let vers = ver.split('.');
        for (i, dig) in vers.into_iter().enumerate() {
            let dignum = dig
                .parse::<u32>()
                .unwrap_or_else(|_| panic!("Invalid browser specifier: '{dig}'"));

            let shift = 16 - 8 * i;
            vercode |= dignum << shift;
        }
        vercode
    }
}

impl From<SabryLightTargets> for lightningcss::targets::Browsers {
    fn from(value: SabryLightTargets) -> Self {
        Self {
            android: value.android.map(|s| SabryLightTargets::parse_ver(&s)),
            chrome: value.chrome.map(|s| SabryLightTargets::parse_ver(&s)),
            edge: value.edge.map(|s| SabryLightTargets::parse_ver(&s)),
            firefox: value.firefox.map(|s| SabryLightTargets::parse_ver(&s)),
            ie: value.ie.map(|s| SabryLightTargets::parse_ver(&s)),
            ios_saf: value.ios_saf.map(|s| SabryLightTargets::parse_ver(&s)),
            opera: value.opera.map(|s| SabryLightTargets::parse_ver(&s)),
            safari: value.safari.map(|s| SabryLightTargets::parse_ver(&s)),
            samsung: value.samsung.map(|s| SabryLightTargets::parse_ver(&s)),
        }
    }
}

/// \[package.metadata.sabry.sass\].module_name_collision option
#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum BehavSassModCollision {
    Merge,
    Error,
}

impl Default for BehavSassModCollision {
    fn default() -> Self {
        Self::Merge
    }
}

/// \[package.metadata.sabry.hash\].collision option
#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum BehavHashCollision {
    Error,
    Ignore,
}

impl Default for BehavHashCollision {
    fn default() -> Self {
        Self::Ignore
    }
}

#[cfg(test)]
mod test {
    use super::{manifest, SabryConfig};

    #[test]
    fn empty_manifest_still_worth_a_read() {
        let manifest = manifest::read_manifest().unwrap();
        let _conf = SabryConfig::from(&manifest);
    }
}

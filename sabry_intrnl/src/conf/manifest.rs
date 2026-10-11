use std::{env, fs, io};

use serde::Deserialize;

use super::SabryConfig;

pub const MANIFEST: &str = "Cargo.toml";
pub const MANIFEST_DIR_ENV: &str = "CARGO_MANIFEST_DIR";

pub fn read_manifest() -> Result<ValuableManifest, ManifestError> {
    let manpath = format!("{}/{MANIFEST}", env::var(MANIFEST_DIR_ENV)?);

    Ok(toml::de::from_str(&fs::read_to_string(manpath)?)?)
}

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("Could not read manifest path from env")]
    ManifestDirEnv(#[from] env::VarError),
    #[error("Could not read manifest file")]
    ManifestRead(#[from] io::Error),
    #[error("Could not deserialize manifest")]
    ManifestDe(#[from] toml::de::Error),
}

/// Manifest structure that makes sense for sabry
#[derive(Deserialize, Default)]
pub struct ValuableManifest {
    pub package: Option<ValuableManifestPkg>,
}

/// Manifest \[package.metadata\] structure that makes sense for sabry
#[derive(Deserialize, Default)]
pub struct ValuableManifestPkg {
    pub metadata: Option<ValuableManifestPkgMeta>,
}

/// Manifest \[package.metadata.sabry\] structure that makes sense for sabry
///
/// Which is basically [SabryConfig]
#[derive(Deserialize, Default)]
pub struct ValuableManifestPkgMeta {
    pub sabry: Option<SabryConfig>,
}

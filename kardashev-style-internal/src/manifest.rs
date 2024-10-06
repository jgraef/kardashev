use std::path::{
    Path,
    PathBuf,
};

use serde::Deserialize;

use crate::Error;

#[derive(Clone, Debug, Deserialize)]
struct Manifest {
    package: Package,
}

#[derive(Clone, Debug, Deserialize)]
struct Package {
    #[serde(default)]
    metadata: Metadata,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct Metadata {
    #[serde(default)]
    kardashev: KardashevMetadata,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct KardashevMetadata {
    #[serde(default)]
    style: StyleMetadata,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StyleMetadata {
    pub output: Option<PathBuf>,
    pub crate_name: Option<String>,
}

impl Default for StyleMetadata {
    fn default() -> Self {
        Self {
            output: None,
            crate_name: None,
        }
    }
}

impl StyleMetadata {
    pub fn read(manifest_path: &Path) -> Result<Self, Error> {
        let toml = std::fs::read_to_string(manifest_path).map_err(|source| {
            Error::ReadManifest {
                source,
                path: manifest_path.to_owned(),
            }
        })?;
        let manifest: Manifest = toml::from_str(&toml).map_err(|source| {
            Error::ParseManifest {
                source,
                path: manifest_path.to_owned(),
            }
        })?;
        Ok(manifest.package.metadata.kardashev.style)
    }
}

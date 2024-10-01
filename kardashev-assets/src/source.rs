use std::{
    collections::HashMap,
    path::PathBuf,
};

use kardashev_protocol::assets::AssetId;
use serde::Deserialize;

use crate::atlas::AtlasBuilderId;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    #[serde(default)]
    pub textures: HashMap<AssetId, Texture>,

    #[serde(default)]
    pub materials: HashMap<AssetId, Material>,

    #[serde(default)]
    pub meshes: HashMap<AssetId, Mesh>,

    #[serde(default)]
    pub models: HashMap<AssetId, Model>,

    #[serde(default)]
    pub sounds: HashMap<AssetId, Sound>,

    #[serde(default)]
    pub shaders: HashMap<AssetId, Shader>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mesh {
    pub label: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Texture {
    pub label: Option<String>,
    pub path: PathBuf,
    pub atlas: Option<AtlasDef>,
    pub convert_to: Option<ImageFormat>,
    pub scale_to: Option<ScaleTo>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum AtlasDef {
    Flag(bool),
    Named(String),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    Jpg,
    Png,
    Gif,
    Webp,
    Tif,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScaleTo {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub filter: Option<ScaleFilter>,
}

#[derive(Copy, Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScaleFilter {
    Nearest,
    #[default]
    Triangle,
    CatmullRom,
    Gaussian,
    Lanczos3,
}

impl From<ScaleFilter> for image::imageops::FilterType {
    fn from(value: ScaleFilter) -> Self {
        match value {
            ScaleFilter::Nearest => Self::Nearest,
            ScaleFilter::Triangle => Self::Triangle,
            ScaleFilter::CatmullRom => Self::CatmullRom,
            ScaleFilter::Gaussian => Self::Gaussian,
            ScaleFilter::Lanczos3 => Self::Lanczos3,
        }
    }
}

impl Default for AtlasDef {
    fn default() -> Self {
        Self::Flag(true)
    }
}

impl From<AtlasDef> for Option<AtlasBuilderId> {
    fn from(value: AtlasDef) -> Self {
        match value {
            AtlasDef::Flag(false) => None,
            AtlasDef::Flag(true) => Some(AtlasBuilderId::Default),
            AtlasDef::Named(name) => Some(AtlasBuilderId::Named(name)),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Material {
    pub label: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    pub label: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sound {
    pub label: Option<String>,
    pub path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum TextureRef {
    Ident(String),
    Texture { texture: String },
    Path { path: PathBuf },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shader {
    pub label: Option<String>,
    pub path: PathBuf,
}

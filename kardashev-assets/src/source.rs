use std::path::PathBuf;

use kardashev_protocol::assets::AssetId;
use serde::Deserialize;

use crate::atlas::AtlasBuilderId;

#[derive(Debug, Deserialize)]
pub struct Manifest {
    #[serde(default)]
    pub texture: Vec<Asset<Texture>>,

    #[serde(default)]
    pub material: Vec<Asset<Material>>,

    #[serde(default)]
    pub mesh: Vec<Asset<Mesh>>,

    #[serde(default)]
    pub model: Vec<Asset<Model>>,

    #[serde(default)]
    pub sound: Vec<Asset<Sound>>,
}

#[derive(Debug, Deserialize)]
pub struct Asset<T> {
    pub id: AssetId,

    pub label: Option<String>,

    #[serde(flatten)]
    pub inner: T,
}

#[derive(Debug, Deserialize)]
pub struct Mesh {}

#[derive(Debug, Deserialize)]
pub struct Texture {
    pub path: Option<PathBuf>,
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
#[serde(untagged)]
pub enum Material {
    Textures {
        diffuse: Option<TextureRef>,
        // todo
    },
}

#[derive(Debug, Deserialize)]
pub struct Model {}

#[derive(Debug, Deserialize)]
pub struct Sound {
    pub path: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TextureRef {
    Ident(String),
    Texture { texture: String },
    Path { path: PathBuf },
}

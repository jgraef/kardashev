use std::{
    collections::HashMap,
    path::PathBuf,
};

use kardashev_protocol::assets::{
    AssetId,
    TextureFormat,
};
use palette::Srgb;
use serde::{
    Deserialize,
    Serialize,
};

use crate::assets::atlas::AtlasBuilderId;

#[derive(Clone, Debug, Default, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mesh {
    pub label: Option<String>,
    pub mesh: PathBuf,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Texture {
    pub label: Option<String>,
    pub path: PathBuf,
    pub atlas: Option<AtlasDef>,
    pub format: Option<TextureFormat>,
    pub output_format: Option<TextureFileFormat>,
    pub scale_to: Option<ScaleTo>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextureFileFormat {
    #[serde(alias = "jpg")]
    Jpeg,
    #[default]
    Png,
    Gif,
    Webp,
    #[serde(alias = "tif")]
    Tiff,
    Ktx2,
}

impl TextureFileFormat {
    pub fn file_extension(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::Gif => "gif",
            Self::Webp => "webp",
            Self::Tiff => "tif",
            Self::Ktx2 => "ktx",
        }
    }

    pub fn image_format(&self) -> Option<image::ImageFormat> {
        match self {
            Self::Jpeg => Some(image::ImageFormat::Jpeg),
            Self::Png => Some(image::ImageFormat::Png),
            Self::Gif => Some(image::ImageFormat::Gif),
            Self::Webp => Some(image::ImageFormat::WebP),
            Self::Tiff => Some(image::ImageFormat::Tiff),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum AtlasDef {
    Flag(bool),
    Named(String),
}

impl Default for AtlasDef {
    fn default() -> Self {
        Self::Flag(false)
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

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Material {
    pub label: Option<String>,
    pub normal: Option<AssetIdOrInline<Texture>>,
    pub ambient: Option<MaterialColorTexture>,
    pub diffuse: Option<MaterialColorTexture>,
    pub specular: Option<MaterialColorTexture>,
    pub shininess: Option<MaterialScalarTexture>,
    pub dissolve: Option<MaterialScalarTexture>,
    pub emissive: Option<MaterialColorTexture>,
    pub albedo: Option<AssetIdOrInline<Texture>>,
    pub metalness: Option<AssetIdOrInline<Texture>>,
    pub roughness: Option<AssetIdOrInline<Texture>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    pub label: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sound {
    pub label: Option<String>,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shader {
    pub label: Option<String>,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum AssetIdOrInline<T> {
    AssetId(AssetId),
    Inline(T),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialColorTexture {
    #[serde(alias = "color")]
    pub tint: Option<Srgb<f32>>,
    pub texture: Option<AssetIdOrInline<Texture>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialScalarTexture {
    pub value: Option<f32>,
    pub texture: Option<AssetIdOrInline<Texture>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaterialProperty {
    Normal,
    Ambient,
    Diffuse,
    Specular,
    Shininess,
    Dissolve,
    Emissive,
    Albedo,
    Metalness,
    Roughness,
}

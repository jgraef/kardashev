use std::fmt::Display;

use serde::{
    Deserialize,
    Serialize,
};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct AssetId(pub Uuid);

impl Display for AssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub textures: Vec<Texture>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub materials: Vec<Material>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mesh: Vec<Mesh>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Texture {
    pub id: AssetId,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    pub image: String,

    pub size: TextureSize,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub crop: Option<TextureCrop>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub u_edge_mode: Option<TextureEdgeMode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub v_edge_mode: Option<TextureEdgeMode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextureSize {
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextureCrop {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TextureEdgeMode {
    ClampToEdge,
    Repeat,
    MirrorRepeat,
    ClampToBorder,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Material {
    pub id: AssetId,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ambient: Option<AssetId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub diffuse: Option<AssetId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub specular: Option<AssetId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub normal: Option<AssetId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub shininess: Option<AssetId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dissolve: Option<AssetId>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mesh {
    pub id: AssetId,

    pub mesh: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Reload { asset_id: AssetId },
}
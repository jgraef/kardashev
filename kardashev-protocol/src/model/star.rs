use nalgebra::Point3;
use palette::LinSrgb;
use serde::{
    Deserialize,
    Serialize,
};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StarId(pub Uuid);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogIds {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hyg: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hip: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hd: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hr: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gl: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bf: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Star {
    pub id: StarId,
    pub position: Point3<f32>,
    pub effective_temperature: f32,
    pub color: LinSrgb,
    pub absolute_magnitude: f32,
    pub luminousity: f32,
    pub radius: f32,
    pub mass: f32,
    pub spectral_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub catalog_ids: CatalogIds,
}

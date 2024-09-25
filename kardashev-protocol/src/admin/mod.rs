use nalgebra::Point3;
use palette::LinSrgb;
use serde::{
    Deserialize,
    Serialize,
};

use crate::model::star::{
    CatalogIds,
    StarId,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateStarsRequest {
    pub stars: Vec<CreateStar>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateStarsResponse {
    pub ids: Vec<StarId>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateStar {
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

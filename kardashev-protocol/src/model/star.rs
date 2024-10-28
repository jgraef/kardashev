use nalgebra::Point3;
use palette::Srgb;
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

    /// in parsec
    /// +X is in the direction of the vernal equinox (at epoch 2000), +Z towards
    /// the north celestial pole, and +Y in the direction of R.A. 6 hours,
    /// declination 0 degrees.
    pub position: Point3<f32>,

    /// effective temperature in Kelvin, derived from luminosity and radius
    pub effective_temperature: f32,

    /// color derived from effective temperature
    pub color: Srgb,

    /// absolute magnitude
    pub absolute_magnitude: f32,

    /// luminosity in multiples of solar luminosity
    pub luminosity: f32,

    /// radius in multiples of solar radii, derived from mass
    pub radius: f32,

    /// mass in multiples of solar mass, derived from luminosity
    pub mass: f32,

    pub spectral_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub catalog_ids: CatalogIds,
}

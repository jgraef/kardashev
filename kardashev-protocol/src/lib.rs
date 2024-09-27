pub mod admin;
pub mod assets;
pub mod model;

use chrono::{
    DateTime,
    Utc,
};
use semver::Version;
use serde::{
    Deserialize,
    Serialize,
};

use crate::model::star::Star;

pub const PROTOCOL_VERSION: Version = semver_macro::version!("0.1.0");

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerStatus {
    pub server_version: Version,
    pub up_since: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetStarsResponse {
    pub stars: Vec<Star>,
}

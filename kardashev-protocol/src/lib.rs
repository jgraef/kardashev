use chrono::{
    DateTime,
    Utc,
};
use semver::Version;
use serde::{
    Deserialize,
    Serialize,
};

pub mod admin;
pub mod model;

pub const PROTOCOL_VERSION: Version = semver_macro::version!("0.1.0");

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerStatus {
    pub server_version: Version,
    pub up_since: DateTime<Utc>,
}

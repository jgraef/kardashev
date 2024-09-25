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
    pub hyg: Option<u32>,
    pub hip: Option<u32>,
    pub hd: Option<u32>,
    pub hr: Option<u32>,
    pub gl: Option<String>,
    pub bf: Option<String>,
}

use leptos::{
    provide_context,
    SignalGetUntracked,
};
use leptos_use::storage::use_local_storage;
use serde::{
    Deserialize,
    Serialize,
};
use url::Url;

use crate::graphics;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Urls {
    pub api_url: Url,
    pub asset_url: Url,
}

impl Default for Urls {
    fn default() -> Self {
        fn get_base_url() -> Option<Url> {
            gloo_utils::document().base_uri().ok()??.parse().ok()
        }
        let base_url: Url = get_base_url().expect("could not determine base URL");
        let api_url = base_url.join("api").unwrap();
        let asset_url = base_url.join("assets").unwrap();
        tracing::debug!(%api_url, %asset_url);
        Urls { api_url, asset_url }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub graphics: graphics::Config,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub urls: Option<Urls>,
}

pub fn provide_config() {
    let (config, _set_config, _delete_config) =
        use_local_storage::<Config, codee::string::JsonSerdeCodec>("graphics-config");
    let config = config.get_untracked();
    provide_context(config)
}

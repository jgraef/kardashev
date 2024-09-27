use std::sync::Arc;

use futures_util::TryStreamExt;
use kardashev_protocol::assets::{
    Manifest,
    Message,
};
use reqwest_websocket::{
    RequestBuilderExt,
    WebSocket,
};
use url::Url;

use crate::{
    Error,
    UrlExt,
};

pub struct AssetClient {
    client: reqwest::Client,
    asset_url: Arc<Url>,
}

impl AssetClient {
    pub fn new(asset_url: Url) -> Self {
        let client = reqwest::Client::new();
        Self {
            client,
            asset_url: Arc::new(asset_url),
        }
    }

    pub async fn get_manifest(&self) -> Result<Manifest, Error> {
        let manifest = self
            .client
            .get(Url::clone(&self.asset_url).joined("assets.json"))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(manifest)
    }

    pub async fn events(&self) -> Result<Events, Error> {
        let websocket = self
            .client
            .get(Url::clone(&self.asset_url).joined("events"))
            .upgrade()
            .send()
            .await?
            .into_websocket()
            .await?;
        Ok(Events { websocket })
    }
}

#[derive(Debug)]
pub struct Events {
    websocket: WebSocket,
}

impl Events {
    pub async fn next(&mut self) -> Result<Message, Error> {
        let message = self
            .websocket
            .try_next()
            .await?
            .ok_or_else(|| Error::UnexpectedEof)?;
        Ok(message.json()?)
    }
}

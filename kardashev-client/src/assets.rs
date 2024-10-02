use std::sync::Arc;

use bytes::Bytes;
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

#[derive(Clone, Debug)]
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

    pub async fn download_file(&self, url: &str) -> Result<DownloadFile, DownloadError> {
        let url = self.asset_url.join(url).expect("invalid url");

        let err = |e| {
            DownloadError {
                url: url.clone(),
                reason: e,
            }
        };

        let response = self
            .client
            .get(url.clone())
            .send()
            .await
            .map_err(err)?
            .error_for_status()
            .map_err(err)?;
        Ok(DownloadFile { url, response })
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

#[derive(Debug)]
pub struct DownloadFile {
    url: Url,
    response: reqwest::Response,
}

impl DownloadFile {
    pub async fn bytes(self) -> Result<Bytes, DownloadError> {
        Ok(self.response.bytes().await.map_err(|e| {
            DownloadError {
                url: self.url.clone(),
                reason: e,
            }
        })?)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("download error: {url}")]
pub struct DownloadError {
    pub url: Url,
    #[source]
    pub reason: reqwest::Error,
}

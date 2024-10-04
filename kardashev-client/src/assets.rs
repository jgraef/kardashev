use std::sync::Arc;

use bytes::{
    Bytes,
    BytesMut,
};
use futures_util::TryStreamExt;
use kardashev_protocol::assets::{
    Event,
    Manifest,
};
use reqwest_websocket::{
    RequestBuilderExt,
    WebSocket,
};
use tokio::sync::watch;
use url::Url;

use crate::{
    add_trailing_slash,
    Error,
    UrlExt,
};

#[derive(Clone, Debug)]
pub struct AssetClient {
    client: reqwest::Client,
    asset_url: Arc<Url>,
}

impl AssetClient {
    pub fn new(mut asset_url: Url) -> Self {
        let client = reqwest::Client::new();

        // the trailing slash is important for `Url::join` to work properly
        add_trailing_slash(&mut asset_url);

        Self {
            client,
            asset_url: Arc::new(asset_url),
        }
    }

    pub fn asset_url(&self) -> &Url {
        &self.asset_url
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
        tracing::debug!(%url, "downloading file");

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

        let content_length = response
            .content_length()
            .and_then(|content_length| usize::try_from(content_length).ok());

        let (tx_progress, _rx_progress) = watch::channel(DownloadProgress {
            total: content_length,
            received: 0,
        });

        Ok(DownloadFile {
            url,
            response,
            tx_progress,
            content_length,
        })
    }
}

#[derive(Debug)]
pub struct Events {
    websocket: WebSocket,
}

impl Events {
    pub async fn next(&mut self) -> Result<Event, Error> {
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
    tx_progress: watch::Sender<DownloadProgress>,
    content_length: Option<usize>,
}

impl DownloadFile {
    pub fn progress(&self) -> watch::Receiver<DownloadProgress> {
        self.tx_progress.subscribe()
    }

    pub async fn bytes(self) -> Result<Bytes, DownloadError> {
        let mut buf = self
            .content_length
            .map(|content_length| BytesMut::with_capacity(content_length))
            .unwrap_or_else(|| BytesMut::new());

        let mut stream = self.response.bytes_stream();

        while let Some(chunk) = stream.try_next().await.map_err(|reason| {
            DownloadError {
                url: self.url.clone(),
                reason,
            }
        })? {
            self.tx_progress.send_modify(|progress| {
                progress.received += chunk.len();
            });

            // can we avoid copying here?
            buf.extend_from_slice(&chunk);
        }

        Ok(buf.freeze())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("download error: {url}")]
pub struct DownloadError {
    pub url: Url,
    #[source]
    pub reason: reqwest::Error,
}

#[derive(Copy, Clone, Debug)]
pub struct DownloadProgress {
    pub total: Option<usize>,
    pub received: usize,
}

mod api;
mod assets;

use url::Url;

pub use crate::{
    api::ApiClient,
    assets::{
        AssetClient,
        DownloadError,
        DownloadFile,
        Events,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("http error")]
    Reqwest(#[from] reqwest::Error),

    #[error("websocket error")]
    Websocket(#[from] reqwest_websocket::Error),

    #[error("unexpected end of stream")]
    UnexpectedEof,
}

trait UrlExt {
    fn joined(self, segment: &str) -> Self;
}

impl UrlExt for Url {
    fn joined(mut self, segment: &str) -> Self {
        let mut segments = self.path_segments_mut().unwrap();
        segments.push(segment);
        drop(segments);
        self
    }
}

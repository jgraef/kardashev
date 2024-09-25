use std::fmt::Display;

use kardashev_protocol::{
    admin::{
        CreateStar,
        CreateStarsRequest,
        CreateStarsResponse,
    },
    model::star::{
        Star,
        StarId,
    },
    GetStarsResponse,
    ServerStatus,
};
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("http error")]
    Reqwest(#[from] reqwest::Error),
}

#[derive(Clone)]
pub struct Client {
    client: reqwest::Client,
    api_url: Url,
}

impl Client {
    pub fn new(api_url: Url) -> Self {
        let client = reqwest::Client::new();
        Self { client, api_url }
    }

    fn url(&self) -> UrlBuilder {
        UrlBuilder {
            url: self.api_url.clone(),
        }
    }

    pub async fn status(&self) -> Result<ServerStatus, Error> {
        let status: ServerStatus = self
            .client
            .get(self.url().add("status").build())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(status)
    }

    pub async fn create_stars(&self, stars: Vec<CreateStar>) -> Result<Vec<StarId>, Error> {
        let response: CreateStarsResponse = self
            .client
            .post(self.url().add("admin").add("star").build())
            .json(&CreateStarsRequest { stars })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(response.ids)
    }

    pub async fn get_stars(&self) -> Result<Vec<Star>, Error> {
        let response: GetStarsResponse = self
            .client
            .get(self.url().add("star").build())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(response.stars)
    }
}

struct UrlBuilder {
    url: Url,
}

impl UrlBuilder {
    pub fn add(mut self, segment: impl Display) -> Self {
        let mut segments = self.url.path_segments_mut().unwrap();
        segments.push(&segment.to_string());
        drop(segments);
        self
    }

    pub fn build(self) -> Url {
        self.url
    }
}

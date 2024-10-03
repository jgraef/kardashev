use std::sync::Arc;

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

use crate::{
    add_trailing_slash,
    Error,
    UrlExt,
};

#[derive(Clone, Debug)]
pub struct ApiClient {
    client: reqwest::Client,
    api_url: Arc<Url>,
}

impl ApiClient {
    pub fn new(mut api_url: Url) -> Self {
        let client = reqwest::Client::new();

        // the trailing slash is important for `Url::join` to work properly
        add_trailing_slash(&mut api_url);

        Self {
            client,
            api_url: Arc::new(api_url),
        }
    }

    pub async fn status(&self) -> Result<ServerStatus, Error> {
        let status: ServerStatus = self
            .client
            .get(Url::clone(&self.api_url).joined("status"))
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
            .post(Url::clone(&self.api_url).joined("admin").joined("star"))
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
            .get(Url::clone(&self.api_url).joined("star"))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(response.stars)
    }
}

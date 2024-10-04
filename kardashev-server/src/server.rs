use std::{
    net::SocketAddr,
    ops::{
        Deref,
        DerefMut,
    },
    path::PathBuf,
    time::Duration,
};

use axum::{
    extract::{
        MatchedPath,
        Request,
    },
    Router,
};
use chrono::{
    DateTime,
    Utc,
};
use sqlx::{
    PgPool,
    Postgres,
};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tower::ServiceBuilder;
use tower_http::trace::{
    DefaultOnRequest,
    DefaultOnResponse,
    TraceLayer,
};

use crate::{
    api,
    error::Error,
    util::ui::UiConfig,
};

#[derive(Clone, Debug)]
pub struct Config {
    assets: Option<kardashev_assets::server::Config>,
    ui: Option<UiConfig>,
}

impl Config {
    pub fn cargo() -> Self {
        let workspace_path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join("..");

        let ui_source_path = workspace_path
            .join("kardashev-ui")
            .canonicalize()
            .expect("Could not get absolute path for UI");

        let ui_dist_path = workspace_path
            .join("kardashev-ui")
            .join("dist")
            .canonicalize()
            .expect("Could not get absolute path for UI");

        let asset_source_path = workspace_path
            .join("assets")
            .join("source")
            .canonicalize()
            .expect("Could not get absolute path for assets");

        let asset_dist_path = workspace_path
            .join("assets")
            .join("dist")
            .canonicalize()
            .expect("Could not get absolute path for assets");

        Self {
            assets: Some(kardashev_assets::server::Config {
                source_path: asset_source_path,
                dist_path: asset_dist_path,
                watch: Some(kardashev_assets::server::WatchConfig {
                    debounce: Duration::from_secs(1),
                }),
            }),
            ui: Some(UiConfig::Trunk {
                source_path: ui_source_path,
                dist_path: ui_dist_path,
            }),
        }
    }

    pub fn api_only() -> Self {
        Self {
            assets: None,
            ui: None,
        }
    }
}

#[derive(Debug)]
pub struct Server {
    router: Router,
    pub shutdown: CancellationToken,
}

impl Server {
    pub async fn new(db: PgPool, config: Config) -> Result<Self, Error> {
        sqlx::migrate!("../migrations").run(&db).await?;

        let shutdown = CancellationToken::new();
        let mut rebuild_assets = None;
        let mut router = Router::new().nest("/api/v0", api::router());

        if let Some(asset_config) = &config.assets {
            let asset_server =
                kardashev_assets::server::Server::new(asset_config, shutdown.clone()).await?;
            router = router.nest_service("/assets", asset_server.router);
            rebuild_assets = Some(asset_server.trigger);
        }

        if let Some(ui_config) = &config.ui {
            router = router.fallback_service(crate::util::ui::router(ui_config).await?);
        }

        let router = router
            .layer(
                ServiceBuilder::new().layer(
                    TraceLayer::new_for_http()
                        .make_span_with(|req: &Request| {
                            let method = req.method();
                            let uri = req.uri();

                            // axum automatically adds this extension.
                            let matched_path = req
                                .extensions()
                                .get::<MatchedPath>()
                                .map(|matched_path| matched_path.as_str());

                            tracing::info_span!("request", %method, %uri, matched_path)
                        })
                        .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                        .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
                ),
            )
            .with_state(Context {
                shutdown: shutdown.clone(),
                db,
                up_since: Utc::now(),
                rebuild_assets,
            });

        Ok(Self { router, shutdown })
    }

    pub async fn bind(self, address: SocketAddr) -> Result<(), Error> {
        tracing::info!("Listening at http://{address}");
        let listener = TcpListener::bind(address).await?;
        axum::serve(listener, self.router)
            .with_graceful_shutdown(async move { self.shutdown.cancelled().await })
            .await?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct Context {
    pub shutdown: CancellationToken,
    pub up_since: DateTime<Utc>,
    pub rebuild_assets: Option<kardashev_assets::server::Trigger>,
    db: PgPool,
}

impl Context {
    pub async fn transaction<'a>(&'a self) -> Result<Transaction<'a>, Error> {
        let transaction = self.db.begin().await?;

        Ok(Transaction { transaction })
    }
}

pub struct Transaction<'a> {
    transaction: sqlx::Transaction<'a, Postgres>,
}

impl<'a> Deref for Transaction<'a> {
    type Target = sqlx::Transaction<'a, Postgres>;

    fn deref(&self) -> &Self::Target {
        &self.transaction
    }
}

impl<'a> DerefMut for Transaction<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transaction
    }
}

impl<'a> Transaction<'a> {
    pub async fn commit(self) -> Result<(), Error> {
        self.transaction.commit().await?;
        Ok(())
    }

    pub async fn rollback(self) -> Result<(), Error> {
        self.transaction.rollback().await?;
        Ok(())
    }
}

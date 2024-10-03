use std::{
    net::SocketAddr,
    ops::{
        Deref,
        DerefMut,
    },
    path::PathBuf,
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
use tower_http::{
    services::{
        ServeDir,
        ServeFile,
    },
    trace::{
        DefaultOnRequest,
        DefaultOnResponse,
        TraceLayer,
    },
};

use crate::{
    api,
    assets::AssetConfig,
    error::Error,
};

#[derive(Clone, Debug)]
pub struct Config {
    assets: Option<AssetConfig>,
    ui_path: Option<PathBuf>,
}

impl Config {
    pub fn cargo() -> Self {
        let workspace_path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join("..");

        let ui_path = workspace_path
            .join("kardashev-ui")
            .join("dist")
            .canonicalize()
            .expect("Could not get absolute path for UI");

        let source_path = workspace_path
            .join("assets")
            .join("dist")
            .canonicalize()
            .expect("Could not get absolute path for assets");

        let dist_path = workspace_path
            .join("assets")
            .join("dist")
            .canonicalize()
            .expect("Could not get absolute path for assets");

        Self {
            assets: Some(AssetConfig {
                source_path,
                dist_path,
            }),
            ui_path: Some(ui_path),
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

        let mut router = Router::new().nest("/api/v0", api::router());

        if let Some(asset_config) = &config.assets {
            router = router.nest_service(
                "/assets",
                crate::assets::router(asset_config, shutdown.clone())?,
            );
        }

        if let Some(ui_path) = &config.ui_path {
            router = router.fallback_service(ServeDir::new(&ui_path).fallback(
                ServeFile::new_with_mime(ui_path.join("index.html"), &mime::TEXT_HTML_UTF_8),
            ));
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
    db: PgPool,
}

impl Context {
    pub fn shutdown(&self) {
        self.shutdown.cancel();
    }
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

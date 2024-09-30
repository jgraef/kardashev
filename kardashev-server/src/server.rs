use std::{
    net::SocketAddr,
    ops::{
        Deref,
        DerefMut,
    },
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
    util::serve_files::{
        serve_assets,
        serve_ui,
    },
};

pub struct Server {
    router: Router,
    shutdown: CancellationToken,
}

impl Server {
    pub async fn new(db: PgPool) -> Result<Self, Error> {
        sqlx::migrate!("../migrations").run(&db).await?;

        let serve_ui = serve_ui()?;
        let serve_assets = serve_assets()?;

        let router = Router::new()
            .nest("/api/v0", api::router())
            .nest_service("/assets", serve_assets)
            .fallback_service(serve_ui)
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
                db,
                up_since: Utc::now(),
            });

        let shutdown = CancellationToken::new();

        Ok(Self { router, shutdown })
    }

    #[must_use]
    pub fn shutdown_token(&self) -> CancellationToken {
        self.shutdown.clone()
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
    db: PgPool,
    pub up_since: DateTime<Utc>,
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

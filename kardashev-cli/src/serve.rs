use std::net::SocketAddr;

use axum::{
    extract::{
        MatchedPath,
        Request,
    },
    Router,
};
use tokio::net::TcpListener;
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
    build::BuildOptions,
    util::shutdown::GracefulShutdown,
    Error,
};

/// Serve API, and optionally assets and UI.
#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(flatten)]
    build_options: BuildOptions,

    /// The address on which to listen for HTTP connections.
    #[arg(long, env = "ADDRESS", default_value = "127.0.0.1:3333")]
    address: SocketAddr,

    /// URL to the server's postgresql database.
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,
}

impl Args {
    pub async fn run(self) -> Result<(), Error> {
        let mut shutdown = GracefulShutdown::new();

        self.build_options.spawn(&mut shutdown).await?;

        let mut router = Router::new().nest(
            "/api",
            kardashev_server::Builder::default()
                .with_shutdown(shutdown.token())
                .with_connect_db(&self.database_url)
                .await?
                .build(),
        );

        if self.build_options.assets {
            let dist_assets = self.build_options.dist_path.join("assets");
            router = router.nest_service("/assets", ServeDir::new(&dist_assets));
        }

        if self.build_options.ui {
            let dist_ui = self.build_options.dist_path.join("ui");
            router = router.fallback_service(ServeDir::new(&dist_ui).fallback(
                ServeFile::new_with_mime(dist_ui.join("index.html"), &mime::TEXT_HTML_UTF_8),
            ));
        }

        router = router.layer(
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
        );

        shutdown.spawn({
            let token = shutdown.token();
            async move {
                tracing::info!("Listening at http://{}", self.address);
                let listener = TcpListener::bind(&self.address).await?;
                axum::serve(listener, router)
                    .with_graceful_shutdown(async move { token.cancelled().await })
                    .await?;
                Ok::<(), Error>(())
            }
        });

        shutdown.join().await
    }
}

use std::net::SocketAddr;

use axum::{
    extract::{
        MatchedPath,
        Request,
    },
    Router,
};
use tokio::{
    net::TcpListener,
    task::JoinSet,
};
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
    build::BuildOptions,
    util::shutdown::graceful_shutdown,
    Error,
};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(flatten)]
    build_options: BuildOptions,

    #[arg(long, env = "ADDRESS", default_value = "127.0.0.1:3333")]
    address: SocketAddr,

    #[arg(long, env = "DATABASE_URL")]
    database_url: String,
}

impl Args {
    pub async fn run(self) -> Result<(), Error> {
        let shutdown = CancellationToken::new();
        graceful_shutdown(shutdown.clone());
        let mut join_set = JoinSet::new();

        self.build_options
            .spawn(shutdown.clone(), &mut join_set)
            .await?;

        let mut router = Router::new().nest(
            "/api",
            kardashev_server::Builder::default()
                .with_shutdown(shutdown.clone())
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

        tokio::spawn(async move {
            tracing::info!("Listening at http://{}", self.address);
            let listener = TcpListener::bind(&self.address).await?;
            axum::serve(listener, router)
                .with_graceful_shutdown(async move { shutdown.cancelled().await })
                .await?;
            Ok::<(), Error>(())
        });

        while let Some(()) = join_set.join_next().await.transpose()?.transpose()? {}

        Ok(())
    }
}

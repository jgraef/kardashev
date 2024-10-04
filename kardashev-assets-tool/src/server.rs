use std::{
    net::SocketAddr,
    path::PathBuf,
    time::Duration,
};

use kardashev_assets::server::{
    Config,
    WatchConfig,
};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

use crate::Error;

pub async fn server(
    assets: PathBuf,
    dist: PathBuf,
    watch: bool,
    watch_debounce: u64,
    address: SocketAddr,
) -> Result<(), Error> {
    let shutdown = graceful_shutdown();

    let asset_server = kardashev_assets::server::Server::new(
        &Config {
            source_path: assets,
            dist_path: dist,
            watch: watch.then(|| {
                WatchConfig {
                    debounce: Duration::from_millis(watch_debounce),
                }
            }),
        },
        shutdown.clone(),
    )
    .await?;

    tracing::info!("Listening at http://{address}");
    let listener = TcpListener::bind(address).await?;

    axum::serve(listener, asset_server.router)
        .with_graceful_shutdown(async move { shutdown.cancelled().await })
        .await?;

    Ok(())
}

fn graceful_shutdown() -> CancellationToken {
    let shutdown = CancellationToken::new();
    tokio::spawn({
        let shutdown = shutdown.clone();
        async move {
            ctrlc_or_sigterm().await;
            shutdown.cancel();
        }
    });
    shutdown
}

async fn sigterm() {
    #[cfg(unix)]
    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .unwrap()
        .recv()
        .await;

    #[cfg(not(unix))]
    std::future::pending::<()>().await;
}

async fn ctrlc_or_sigterm() {
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received Ctrl-C. Shutting down.");
        },
        _ = sigterm() => {
            tracing::info!("Received SIGTERM. Shutting down.");
        }
    }
}

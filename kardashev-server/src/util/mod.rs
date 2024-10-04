use tokio_util::sync::CancellationToken;

pub mod sqlx;
pub mod ui;

pub fn graceful_shutdown(shutdown: CancellationToken) {
    tokio::spawn(async move {
        ctrlc_or_sigterm().await;
        shutdown.cancel();
    });
}

pub async fn sigterm() {
    #[cfg(unix)]
    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .unwrap()
        .recv()
        .await;

    #[cfg(not(unix))]
    std::future::pending::<()>().await;
}

pub async fn ctrlc_or_sigterm() {
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received Ctrl-C. Shutting down.");
        },
        _ = sigterm() => {
            tracing::info!("Received SIGTERM. Shutting down.");
        }
    }
}

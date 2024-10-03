use std::{
    path::PathBuf,
    time::Duration,
};

use axum::{
    extract::{
        ws::{
            self,
            WebSocket,
        },
        WebSocketUpgrade,
    },
    routing,
    Router,
};
use kardashev_assets::{
    dist,
    process,
    processor::Processed,
};
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use tower_http::services::ServeDir;

use crate::error::Error;

#[derive(Clone, Debug)]
pub struct AssetConfig {
    pub source_path: PathBuf,
    pub dist_path: PathBuf,
}

pub fn router(config: &AssetConfig, shutdown: CancellationToken) -> Result<Router, Error> {
    let mut watch = notify_async::watch_modified(&config.source_path, Duration::from_secs(1))?;
    let (tx, _) = broadcast::channel(128);

    tokio::spawn({
        let source_path = config.source_path.clone();
        let dist_path = config.dist_path.clone();
        let tx = tx.clone();

        async move {
            let processed = process(&source_path, &dist_path).unwrap();
            let _ = tx.send(processed);

            loop {
                tokio::select! {
                    _ = shutdown.cancelled() => break,
                    result = watch.modified() => {
                        result.unwrap();
                        let processed = process(&source_path, &dist_path).unwrap();
                        if tx.send(processed).is_err() {
                            break;
                        }
                    }
                }
            }
        }
    });

    let router = Router::new().route(
        "/events",
        routing::get(move |websocket_upgrade: WebSocketUpgrade| {
            let tx = tx.clone();
            async move {
                websocket_upgrade.on_upgrade(move |websocket| {
                    async move {
                        let rx = tx.subscribe();
                        if let Err(error) = handle_websocket(websocket, rx).await {
                            tracing::error!(?error, "asset event stream failed");
                        }
                    }
                })
            }
        })
        .fallback_service(ServeDir::new(&config.dist_path)),
    );

    Ok(router)
}

pub async fn handle_websocket(
    mut websocket: WebSocket,
    mut rx: broadcast::Receiver<Processed>,
) -> Result<(), Error> {
    while let Ok(processed) = rx.recv().await {
        let json = serde_json::to_string(&dist::Message::Changed {
            asset_ids: processed.changed.into_iter().collect(),
        })
        .unwrap();
        websocket.send(ws::Message::Text(json)).await?;
    }

    Ok(())
}

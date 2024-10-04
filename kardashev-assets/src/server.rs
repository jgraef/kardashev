use std::{
    path::{
        Path,
        PathBuf,
    },
    time::Duration,
};

use axum::{
    extract::{
        ws,
        WebSocketUpgrade,
    },
    routing,
    Router,
};
use serde::{
    Deserialize,
    Serialize,
};
use tokio::sync::{
    broadcast,
    mpsc,
    oneshot,
};
use tokio_util::sync::CancellationToken;
use tower_http::services::ServeDir;

use crate::{
    dist,
    processor::Processed,
    Error,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub source_path: PathBuf,
    pub dist_path: PathBuf,
    pub watch: Option<WatchConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WatchConfig {
    pub debounce: Duration,
}

#[derive(Clone, Debug)]
pub struct Server {
    pub router: Router<()>,
    pub trigger: Trigger,
}

impl Server {
    pub async fn new(config: &Config, shutdown: CancellationToken) -> Result<Self, Error> {
        process(&config.source_path, &config.dist_path).await?;

        let (trigger_tx, mut trigger_rx) = mpsc::channel(16);
        let trigger = Trigger { tx: trigger_tx };
        let (changed_tx, _) = broadcast::channel(32);

        if let Some(watch_config) = &config.watch {
            let mut watch =
                notify_async::watch_modified(&config.source_path, watch_config.debounce)?;
            let shutdown = shutdown.clone();
            let trigger = trigger.clone();

            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = shutdown.cancelled() => break,
                        result = watch.modified() => {
                            result.unwrap();
                            if let Err(error) = trigger.trigger().await {
                                tracing::error!(?error, "error while processing assets");
                            }
                        }
                    }
                }
            });
        }

        tokio::spawn({
            let source_path = config.source_path.clone();
            let dist_path = config.dist_path.clone();
            let changed_tx = changed_tx.clone();

            async move {
                loop {
                    tokio::select! {
                        _ = shutdown.cancelled() => break,
                        result_tx_opt = trigger_rx.recv() => {
                            let Some(result_tx) = result_tx_opt else { break; };
                            let result = process(&source_path, &dist_path).await;
                            if let Ok(processed) = &result {
                                let _ = changed_tx.send(processed.changed.iter().copied().collect());
                            }
                            let _ = result_tx.send(result);
                        }
                    }
                }
            }
        });

        let router = Router::new()
            .route(
                "/events",
                routing::get(move |websocket_upgrade: WebSocketUpgrade| {
                    let changed_tx = changed_tx.clone();
                    async move {
                        websocket_upgrade.on_upgrade(move |mut websocket| {
                            async fn send_message(
                                websocket: &mut ws::WebSocket,
                                message: &dist::Event,
                            ) -> Result<(), Error> {
                                let json = serde_json::to_string(&message).unwrap();
                                websocket.send(ws::Message::Text(json)).await?;
                                Ok(())
                            }

                            async move {
                                let mut changed_rx = changed_tx.subscribe();

                                loop {
                                    match changed_rx.recv().await {
                                        Ok(changed_assets) => {
                                            if let Err(_error) = send_message(
                                                &mut websocket,
                                                &dist::Event::Changed {
                                                    asset_ids: changed_assets,
                                                },
                                            )
                                            .await
                                            {
                                                break;
                                            }
                                        }
                                        Err(broadcast::error::RecvError::Closed) => break,
                                        Err(broadcast::error::RecvError::Lagged(_)) => {
                                            if let Err(_error) =
                                                send_message(&mut websocket, &dist::Event::Lagged)
                                                    .await
                                            {
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        })
                    }
                }),
            )
            .fallback_service(ServeDir::new(&config.dist_path));

        Ok(Self { router, trigger })
    }
}

#[derive(Clone, Debug)]
pub struct Trigger {
    tx: mpsc::Sender<oneshot::Sender<Result<Processed, Error>>>,
}

impl Trigger {
    pub async fn trigger(&self) -> Result<Processed, Error> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(tx).await.unwrap();
        rx.await.unwrap()
    }
}

async fn process(
    source_path: impl AsRef<Path>,
    dist_path: impl AsRef<Path>,
) -> Result<Processed, Error> {
    tracing::info!("processing assets");

    let source_path = source_path.as_ref().to_owned();
    let dist_path = dist_path.as_ref().to_owned();

    let processed = tokio::task::spawn_blocking(move || crate::process(&source_path, &dist_path))
        .await
        .unwrap()?;

    Ok(processed)
}

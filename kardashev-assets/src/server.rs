use std::{
    path::PathBuf,
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
    processor::{Processed, Processor},
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
        crate::process(&config.source_path, &config.dist_path, false).await?;

        let (trigger_tx, mut trigger_rx) = mpsc::channel(16);
        let trigger = Trigger { tx: trigger_tx };
        let (changed_tx, _) = broadcast::channel(32);

        let mut processor = Processor::new(&config.dist_path)?;
        processor.add_directory(&config.source_path)?;
        let debounce = config.watch.as_ref().map(|watch_config| watch_config.debounce);

        tokio::spawn({
            let changed_tx = changed_tx.clone();

            async move {
                loop {
                    tokio::select! {
                        _ = shutdown.cancelled() => break,
                        changed_paths_opt = processor.wait_for_changes(debounce) => {
                            let Some(_changed_paths) = changed_paths_opt else { break; };
                            if let Err(error) = processor.process(false).await {
                                tracing::error!(?error, "asset processor failed");
                            }
                            
                        }
                        result_tx_opt = trigger_rx.recv() => {
                            let Some(result_tx) = result_tx_opt else { break; };
                            let result = processor.process(false).await;
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

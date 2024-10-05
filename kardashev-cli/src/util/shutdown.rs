use std::future::Future;

use tokio::task::{
    AbortHandle,
    JoinSet,
};
use tokio_util::sync::CancellationToken;

use crate::Error;

#[derive(Debug)]
pub struct GracefulShutdown {
    token: CancellationToken,
    join_set: JoinSet<Result<(), Error>>,
}

impl GracefulShutdown {
    pub fn new() -> Self {
        let token = CancellationToken::new();

        tokio::spawn({
            let token = token.clone();
            async move {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {
                        tracing::info!("Received Ctrl-C. Shutting down.");
                        token.cancel();
                    },
                    _ = sigterm() => {
                        tracing::info!("Received SIGTERM. Shutting down.");
                        token.cancel();
                    }
                    _ = token.cancelled() => {}
                }
            }
        });

        Self {
            token,
            join_set: JoinSet::new(),
        }
    }

    pub fn token(&self) -> CancellationToken {
        self.token.clone()
    }

    pub fn shutdown(&self) {
        self.token.cancel();
    }

    pub fn spawn<F>(&mut self, future: F) -> AbortHandle
    where
        F: Future<Output = Result<(), Error>> + Send + 'static,
    {
        self.join_set.spawn(future)
    }

    pub async fn join(mut self) -> Result<(), Error> {
        let mut errors = vec![];

        // wait for the cancellation signal, while handling joined tasks
        loop {
            tokio::select! {
                _ = self.token.cancelled() => break,
                opt = self.join_set.join_next() => {
                    match opt {
                        None => break,
                        Some(Err(join_error)) => {
                            errors.push(Error::from(join_error));
                            self.token.cancel();
                            break;
                        }
                        Some(Ok(Err(task_error))) => {
                            errors.push(task_error);
                            self.token.cancel();
                            break;
                        }
                        Some(Ok(Ok(()))) => {},
                    }
                }
            }
        }

        // wait for all remaining tasks to finish. but if a second Ctrl-C/SIGTERM
        // arrives, we abort.
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!("Received second Ctrl-C. Aborting.");
                    break;
                },
                _ = sigterm() => {
                    tracing::info!("Received second SIGTERM. Aborting.");
                    break;
                }
                opt = self.join_set.join_next() => {
                    match opt {
                        None => break,
                        Some(Err(join_error)) => {
                            errors.push(Error::from(join_error));
                        }
                        Some(Ok(Err(task_error))) => {
                            errors.push(task_error);
                        }
                        Some(Ok(Ok(()))) => break,
                    }
                }
            }
        }

        // abort all remaining tasks
        self.join_set.abort_all();

        // log/return errors
        match errors.len() {
            0 => Ok(()),
            1 => Err(errors.pop().unwrap()),
            _ => {
                tracing::error!("Multiple errors occurred:");
                for error in &errors {
                    tracing::error!("{error}");
                }
                Err(errors.pop().unwrap())
            }
        }
    }
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

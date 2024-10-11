use std::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
    time::Duration,
};

use futures::{
    pin_mut,
    FutureExt,
    StreamExt,
};
use tokio::sync::{
    oneshot,
    watch,
};

pub fn spawn_local<F: Future<Output = T> + 'static, T: 'static>(fut: F) -> JoinHandle<T> {
    let (tx_result, rx_result) = oneshot::channel();
    let (tx_cancel, rx_cancel) = watch::channel(false);

    wasm_bindgen_futures::spawn_local(async move {
        pin_mut!(fut);
        let mut rx_cancel = Some(rx_cancel);

        async fn wait_cancelled(rx_cancel_option: &mut Option<watch::Receiver<bool>>) {
            if let Some(rx_cancel) = rx_cancel_option {
                let result = rx_cancel.wait_for(|cancel| *cancel).await;
                if result.is_err() {
                    drop(result);
                    *rx_cancel_option = None;
                    std::future::pending::<()>().await;
                }
            }
            else {
                std::future::pending::<()>().await;
            }
        }

        tokio::select! {
            _ = wait_cancelled(&mut rx_cancel) => {
                tracing::debug!("future cancelled");
                let _ = tx_result.send(Err(JoinError::Cancelled));
            }
            result = &mut fut => {
                let _ = tx_result.send(Ok(result));
            }
        }
    });

    JoinHandle {
        rx_result,
        abort_handle: AbortHandle { tx_cancel },
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JoinError {
    #[error("task was cancelled")]
    Cancelled,
    #[error("task panicked")]
    Panic,
}

#[derive(Debug)]
pub struct JoinHandle<T> {
    rx_result: oneshot::Receiver<Result<T, JoinError>>,
    abort_handle: AbortHandle,
}

impl<T> JoinHandle<T> {
    pub fn abort(&self) {
        self.abort_handle.abort()
    }

    pub fn abort_handle(&self) -> AbortHandle {
        self.abort_handle.clone()
    }

    pub fn is_finished(&self) -> bool {
        self.abort_handle.is_finished()
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = Result<T, JoinError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.rx_result
            .poll_unpin(cx)
            .map(|result| result.unwrap_or(Err(JoinError::Panic)))
    }
}

#[derive(Clone, Debug)]
pub struct AbortHandle {
    tx_cancel: watch::Sender<bool>,
}

impl AbortHandle {
    pub fn abort(&self) {
        let _ = self.tx_cancel.send(true);
    }

    pub fn is_finished(&self) -> bool {
        self.tx_cancel.is_closed()
    }
}

pub fn spawn_local_and_handle_error<
    F: Future<Output = Result<(), E>> + 'static,
    E: std::error::Error,
>(
    fut: F,
) {
    spawn_local(fut.map(|result| {
        if let Err(error) = result {
            let mut error: &dyn std::error::Error = &error;

            tracing::error!(%error);

            while let Some(source) = error.source() {
                tracing::error!(%source);
                error = source;
            }
        }
    }));
}

#[derive(Debug)]
pub struct Interval {
    inner: gloo_timers::future::IntervalStream,
}

impl Interval {
    fn new(period: Duration) -> Self {
        let millis = period.as_millis().try_into().expect("period too long");
        Self {
            inner: gloo_timers::future::IntervalStream::new(millis),
        }
    }

    pub async fn tick(&mut self) {
        self.inner.next().await.expect("interval stream finished");
    }

    pub fn poll_tick(&mut self, cx: &mut Context) -> Poll<()> {
        self.inner
            .poll_next_unpin(cx)
            .map(|option| option.expect("interval stream finished"))
    }
}

/// # Panics
///
/// Panics if the duration is too long. This is the case if the period in
/// milliseconds can not be stored in a 32bit unsigned integer. This is the case
/// for any period longer than 4294967 seconds, or about 50 days.
pub fn interval(period: Duration) -> Interval {
    Interval::new(period)
}

#[derive(Debug)]
pub struct Sleep {
    inner: gloo_timers::future::TimeoutFuture,
}

impl Sleep {
    fn new(duration: Duration) -> Sleep {
        let millis = duration.as_millis().try_into().expect("period too long");
        Self {
            inner: gloo_timers::future::TimeoutFuture::new(millis),
        }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.inner.poll_unpin(cx)
    }
}

/// # Panics
///
/// Panics if the duration is too long. This is the case if the period in
/// milliseconds can not be stored in a 32bit unsigned integer. This is the case
/// for any period longer than 4294967 seconds, or about 50 days.
pub fn sleep(duration: Duration) -> Sleep {
    Sleep::new(duration)
}

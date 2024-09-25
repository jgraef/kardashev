use std::{
    path::{
        Path,
        PathBuf,
    },
    task::{
        Context,
        Poll,
    },
    time::Duration,
};

use axum::{
    body::Body,
    http::Request,
};
use notify_async::watch_modified;
use tower::Service;
use tower_http::services::{
    ServeDir,
    ServeFile,
};

use crate::error::Error;

#[derive(Clone, Debug)]
pub struct ServeFiles {
    inner: ServeDir<ServeFile>,
}

impl ServeFiles {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();

        tracing::info!(path = %path.display(), "serving files");

        let mut watch =
            watch_modified(path, Duration::from_secs(2)).expect("Failed to watch for file changes");
        tokio::spawn(async move {
            while let Ok(()) = watch.modified().await {
                //reload_trigger.trigger();
                // todo
            }
        });

        let inner = ServeDir::new(path).fallback(ServeFile::new_with_mime(
            path.join("index.html"),
            &mime::TEXT_HTML_UTF_8,
        ));

        Self { inner }
    }
}

impl Service<Request<Body>> for ServeFiles {
    type Response = <ServeDir<ServeFile> as Service<Request<Body>>>::Response;
    type Error = <ServeDir<ServeFile> as Service<Request<Body>>>::Error;
    type Future = <ServeDir<ServeFile> as Service<Request<Body>>>::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        <ServeDir<ServeFile> as Service<Request<Body>>>::poll_ready(&mut self.inner, cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        <ServeDir<ServeFile> as Service<Request<Body>>>::call(&mut self.inner, req)
    }
}

pub fn serve_ui() -> Result<ServeFiles, Error> {
    let path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("kardashev-ui")
        .join("dist")
        .canonicalize()
        .expect("Could not get absolute path for UI");

    Ok(ServeFiles::new(path))
}

pub fn serve_assets() -> Result<ServeFiles, Error> {
    let path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("assets")
        .join("dist")
        .canonicalize()
        .expect("Could not get absolute path for assets");

    Ok(ServeFiles::new(path))
}

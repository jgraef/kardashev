use std::{
    path::{
        Path,
        PathBuf,
    },
    process::ExitStatus,
};

use axum::Router;
use tokio::process::Command;
use tower_http::services::{
    ServeDir,
    ServeFile,
};

use crate::error::Error;

#[derive(Clone, Debug)]
pub enum UiConfig {
    Dist {
        dist_path: PathBuf,
    },
    Trunk {
        source_path: PathBuf,
        dist_path: PathBuf,
    },
}

pub async fn router(config: &UiConfig) -> Result<Router, Error> {
    let dist_path = match config {
        UiConfig::Dist { dist_path } => dist_path,
        UiConfig::Trunk {
            source_path,
            dist_path,
        } => {
            tracing::info!("building UI");
            Trunk::new(&source_path, &dist_path).build().await?;
            dist_path
        }
    };

    let router = Router::new().fallback_service(ServeDir::new(&dist_path).fallback(
        ServeFile::new_with_mime(dist_path.join("index.html"), &mime::TEXT_HTML_UTF_8),
    ));

    Ok(router)
}

#[derive(Debug)]
pub struct Trunk {
    source_path: PathBuf,
    dist_path: PathBuf,
}

impl Trunk {
    pub fn new(source_path: impl AsRef<Path>, dist_path: impl AsRef<Path>) -> Self {
        Self {
            source_path: source_path.as_ref().to_owned(),
            dist_path: dist_path.as_ref().to_owned(),
        }
    }

    pub async fn build(&self) -> Result<(), TrunkError> {
        let mut process = Command::new("trunk")
            .current_dir(&self.source_path)
            .arg("build")
            .arg("--dist")
            .arg(&self.dist_path)
            .spawn()?;
        let exit_status = process.wait().await?;
        if exit_status.success() {
            Ok(())
        }
        else {
            Err(TrunkError::ExitStatus(exit_status))
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("trunk error")]
pub enum TrunkError {
    Io(#[from] std::io::Error),
    ExitStatus(ExitStatus),
}

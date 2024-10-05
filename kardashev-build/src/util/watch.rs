use std::{
    collections::HashSet,
    path::{
        Path,
        PathBuf,
    },
    time::Duration,
};

use notify::{
    Event,
    EventKind,
    RecommendedWatcher,
    RecursiveMode,
    Watcher as _,
};
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct WatchSources {
    manifest_paths: HashSet<PathBuf>,
    source_paths: HashSet<PathBuf>,
    watch_files: WatchFiles,
}

impl WatchSources {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            manifest_paths: HashSet::new(),
            source_paths: HashSet::new(),
            watch_files: WatchFiles::new()?,
        })
    }

    pub fn add_manifest_path(&mut self, path: impl AsRef<Path>) -> Result<(), Error> {
        let path = path.as_ref();
        if !self.manifest_paths.contains(path) {
            self.watch_files.watch(path)?;
            self.manifest_paths.insert(path.to_owned());
        }
        Ok(())
    }

    pub fn set_source_paths(&mut self, files: HashSet<PathBuf>) -> Result<(), Error> {
        for path in &self.source_paths {
            if !files.contains(path) {
                self.watch_files.unwatch(path)?;
            }
        }

        for path in &files {
            if !self.source_paths.contains(path) {
                self.watch_files.watch(path)?;
            }
        }

        self.source_paths = files;
        Ok(())
    }

    pub async fn next_changes(&mut self, debounce: Option<Duration>) -> Option<ChangedPaths> {
        self.watch_files.next(debounce).await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("watch event error")]
    Event {
        #[from]
        source: notify::Error,
    },
    #[error("watch error: {path}")]
    Watch {
        #[source]
        source: notify::Error,
        path: PathBuf,
    },
    #[error("unwatch error: {path}")]
    Unwatch {
        #[source]
        source: notify::Error,
        path: PathBuf,
    },
}

#[derive(Debug)]
pub struct WatchFiles {
    watcher: RecommendedWatcher,
    events: mpsc::Receiver<Vec<PathBuf>>,
}

impl WatchFiles {
    pub fn new() -> Result<Self, Error> {
        let (tx, rx) = mpsc::channel(128);

        let watcher =
            notify::recommended_watcher(move |event_result: Result<Event, notify::Error>| {
                match event_result {
                    Ok(event) => {
                        match event.kind {
                            EventKind::Any
                            | EventKind::Create(_)
                            | EventKind::Modify(_)
                            | EventKind::Remove(_) => {
                                let _ = tx.blocking_send(event.paths);
                            }
                            _ => {}
                        }
                    }
                    Err(error) => {
                        tracing::error!(?error, "error while watching files");
                    }
                }
            })?;

        Ok(Self {
            watcher,
            events: rx,
        })
    }

    pub fn watch(&mut self, path: impl AsRef<Path>) -> Result<(), Error> {
        let path = path.as_ref();
        tracing::debug!(path = %path.display(), "watching files");
        self.watcher
            .watch(path, RecursiveMode::Recursive)
            .map_err(|source| {
                Error::Watch {
                    source,
                    path: path.to_owned(),
                }
            })
    }

    pub fn unwatch(&mut self, path: impl AsRef<Path>) -> Result<(), Error> {
        let path = path.as_ref();
        tracing::debug!(path = %path.display(), "unwatching files");
        self.watcher.unwatch(path).map_err(|source| {
            Error::Unwatch {
                source,
                path: path.to_owned(),
            }
        })
    }

    pub async fn next(&mut self, debounce: Option<Duration>) -> Option<ChangedPaths> {
        let mut changed = self
            .events
            .recv()
            .await?
            .into_iter()
            .collect::<HashSet<PathBuf>>();

        tracing::debug!(?changed, "changed");

        if let Some(debounce) = debounce {
            loop {
                match tokio::time::timeout(debounce, self.events.recv()).await {
                    Ok(Some(paths)) => {
                        changed.extend(paths);
                    }
                    Ok(None) | Err(_) => {
                        break;
                    }
                }
            }
        }

        Some(ChangedPaths { paths: changed })
    }
}

#[derive(Clone, Debug)]
pub struct ChangedPaths {
    pub paths: HashSet<PathBuf>,
}

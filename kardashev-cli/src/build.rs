use std::{
    path::PathBuf,
    time::Duration,
};

use kardashev_build::{
    assets::processor::Processor,
    ui::compile_ui,
    util::watch::WatchFiles,
};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

use crate::{
    util::shutdown::graceful_shutdown,
    Error,
};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(flatten)]
    build_options: BuildOptions,
}

impl Args {
    pub async fn run(self) -> Result<(), Error> {
        let shutdown = CancellationToken::new();
        graceful_shutdown(shutdown.clone());
        let mut join_set = JoinSet::new();

        self.build_options.spawn(shutdown, &mut join_set).await?;

        while let Some(()) = join_set.join_next().await.transpose()?.transpose()? {}

        Ok(())
    }
}

#[derive(Debug, clap::Args)]
pub struct BuildOptions {
    #[arg(long = "dist", env = "KARDASHEV_DIST", default_value = "./dist/")]
    pub dist_path: PathBuf,

    #[arg(long)]
    pub assets: bool,

    #[arg(long, env = "KARDASHEV_ASSETS", default_value = "./assets/")]
    pub assets_path: PathBuf,

    #[arg(long)]
    pub ui: bool,

    #[arg(long, env = "KARDASHEV_UI", default_value = "./kardashev-ui/")]
    pub ui_path: PathBuf,

    #[arg(long)]
    pub watch: bool,

    #[arg(long)]
    pub clean: bool,

    #[arg(long, default_value = "2")]
    pub debounce: f32,

    #[arg(long)]
    pub no_debounce: bool,
}

impl BuildOptions {
    pub async fn spawn(
        &self,
        shutdown: CancellationToken,
        join_set: &mut JoinSet<Result<(), Error>>,
    ) -> Result<(), Error> {
        let debounce = (!self.no_debounce).then(|| Duration::from_secs_f32(self.debounce));

        if self.assets {
            let dist_assets = self.dist_path.join("assets");
            let mut processor = Processor::new(&dist_assets)?;
            if self.watch {
                processor.watch_source_files()?;
            }
            processor.add_directory(&self.assets_path)?;
            processor.process(self.clean).await?;

            let shutdown = shutdown.clone();
            join_set.spawn(async move {
                loop {
                    tokio::select! {
                        _ = shutdown.cancelled() => break,
                        changes_option = processor.wait_for_changes(debounce) => {
                            let Some(_changes) = changes_option else { break; };
                            if let Err(error) = processor.process(false).await {
                                tracing::error!(%error);
                            }
                        }
                    }
                }

                Ok(())
            });
        }

        if self.ui {
            let dist_ui = self.dist_path.join("ui");
            compile_ui(&self.ui_path, &dist_ui).await?;
            let ui_path = self.ui_path.clone();
            let mut watch_files = WatchFiles::new()?;

            let shutdown = shutdown.clone();
            join_set.spawn(async move {
                loop {
                    tokio::select! {
                        _ = shutdown.cancelled() => break,
                        changes_option = watch_files.next(debounce) => {
                            let Some(_changes) = changes_option else { break; };
                            if let Err(error) = compile_ui(&ui_path, &dist_ui).await {
                                tracing::error!(%error);
                            }
                        }
                    }
                }

                Ok(())
            });
        }

        Ok(())
    }
}

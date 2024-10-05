use std::{
    path::PathBuf,
    time::Duration,
};

use kardashev_build::{
    assets::processor::Processor,
    ui::compile_ui,
    util::watch::WatchFiles,
};

use crate::{
    util::shutdown::GracefulShutdown,
    Error,
};

/// Build assets and UI.
#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(flatten)]
    build_options: BuildOptions,
}

impl Args {
    pub async fn run(self) -> Result<(), Error> {
        let mut shutdown = GracefulShutdown::new();

        self.build_options.spawn(&mut shutdown).await?;

        shutdown.join().await
    }
}

#[derive(Debug, clap::Args)]
pub struct BuildOptions {
    /// Path to the dist directory. This is where the generated files will be
    /// stored.
    #[arg(long = "dist", env = "KARDASHEV_DIST", default_value = "./dist/")]
    pub dist_path: PathBuf,

    /// Build assets
    #[arg(long)]
    pub assets: bool,

    /// Path to the assets directory. This should contain one or more
    /// `Asset.toml` files.
    #[arg(long, env = "KARDASHEV_ASSETS", default_value = "./assets/")]
    pub assets_path: PathBuf,

    /// Build UI
    #[arg(long)]
    pub ui: bool,

    /// Path to the UI crate.
    #[arg(long, env = "KARDASHEV_UI", default_value = "./kardashev-ui/")]
    pub ui_path: PathBuf,

    /// Watch for file changes.
    #[arg(long)]
    pub watch: bool,

    /// After a file change, wait N seconds before rebuilding to avoid to many
    /// rebuild events.
    #[arg(long, default_value = "2")]
    pub debounce: f32,

    /// Disable debounce.
    #[arg(long)]
    pub no_debounce: bool,

    /// Start with a clean build.
    #[arg(long)]
    pub clean: bool,
}

impl BuildOptions {
    pub async fn spawn(&self, shutdown: &mut GracefulShutdown) -> Result<(), Error> {
        let debounce = (!self.no_debounce).then(|| Duration::from_secs_f32(self.debounce));

        if self.assets {
            let dist_assets = self.dist_path.join("assets");
            let mut processor = Processor::new(&dist_assets)?;
            if self.watch {
                processor.watch_source_files()?;
            }
            processor.add_directory(&self.assets_path)?;
            processor.process(self.clean).await?;

            if self.watch {
                let token = shutdown.token();
                shutdown.spawn(async move {
                    loop {
                        tokio::select! {
                            _ = token.cancelled() => break,
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
        }

        if self.ui {
            let dist_ui = self.dist_path.join("ui");
            compile_ui(&self.ui_path, &dist_ui).await?;

            if self.watch {
                let ui_path = self.ui_path.clone();
                let mut watch_files = WatchFiles::new()?;
                watch_files.watch(&ui_path)?;

                let token = shutdown.token();
                shutdown.spawn(async move {
                    loop {
                        tokio::select! {
                            _ = token.cancelled() => break,
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
        }

        if self.watch {
            tracing::info!("Watching for file changes...");
        }

        Ok(())
    }
}

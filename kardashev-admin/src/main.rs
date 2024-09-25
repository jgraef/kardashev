#![feature(btree_cursors)]

pub mod catalog;
pub mod import_stars;
pub mod utils;

use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::Error;
use kardashev_client::Client;
use url::Url;

use crate::import_stars::import_stars;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long, short, env = "KARDASHEV_API_URL")]
    api_url: Url,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Parser)]
pub enum Command {
    ImportStars {
        path: PathBuf,

        #[arg(long, default_value = "100")]
        batch_size: usize,
    },
}

impl Args {
    async fn run(self) -> Result<(), Error> {
        let api = Client::new(self.api_url);

        let status = api.status().await?;
        tracing::info!(?status);

        if let Some(command) = self.command {
            match command {
                Command::ImportStars { path, batch_size } => {
                    import_stars(&api, path, batch_size).await?
                }
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    args.run().await?;

    Ok(())
}

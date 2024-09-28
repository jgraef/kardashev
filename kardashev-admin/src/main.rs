pub mod catalog;
pub mod import_stars;
pub mod utils;

use std::path::PathBuf;

use chrono::Utc;
use clap::Parser;
use color_eyre::eyre::Error;
use kardashev_client::ApiClient;
use url::Url;
use utils::format_uptime;

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

        #[arg(long)]
        num_closest: Option<usize>,
    },
}

impl Args {
    async fn run(self) -> Result<(), Error> {
        let api = ApiClient::new(self.api_url);

        let status = api.status().await?;
        println!("Server version: {}", status.server_version);
        let uptime = Utc::now() - status.up_since;
        println!(
            "Uptime: {} (since {})",
            format_uptime(uptime),
            status.up_since
        );

        if let Some(command) = self.command {
            match command {
                Command::ImportStars {
                    path,
                    batch_size,
                    num_closest,
                } => import_stars(&api, path, batch_size, num_closest).await?,
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

mod catalog;
mod import_stars;
mod utils;

use std::path::PathBuf;

use chrono::Utc;
use color_eyre::eyre::Error;
use kardashev_client::ApiClient;
use url::Url;
use utils::format_uptime;

use crate::admin::import_stars::import_stars;

/// Send administrative commands to the server API.
#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg(long, short, env = "KARDASHEV_API_URL")]
    api_url: Url,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Import stars into the database.
    ///
    /// Input file must be the same format as the HYG catalog.
    ImportStars {
        /// Input file (HYG catalog)
        path: PathBuf,

        /// How many stars to send to the server in one request.
        #[arg(long, default_value = "100")]
        batch_size: usize,

        /// Only import the N stars that are closest to the sun.
        #[arg(long)]
        num_closest: Option<usize>,
    },
}

impl Args {
    pub async fn run(self) -> Result<(), Error> {
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

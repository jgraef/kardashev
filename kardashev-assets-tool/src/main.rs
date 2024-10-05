#[cfg(feature = "server")]
mod server;

use std::{
    net::SocketAddr,
    path::PathBuf,
};

use clap::Parser;
use color_eyre::eyre::Error;
use kardashev_assets::{
    process,
    AssetId,
};

#[derive(Debug, Parser)]
pub enum Args {
    Id {
        #[arg(short, default_value = "1")]
        n: usize,
    },
    Build {
        #[arg(long, env = "ASSETS", default_value = "./assets/source")]
        assets: PathBuf,

        #[arg(long, env = "DIST", default_value = "./assets/dist")]
        dist: PathBuf,

        #[arg(long)]
        clean: bool,
    },
    #[cfg(feature = "server")]
    Serve {
        #[arg(long, env = "ASSETS", default_value = "./assets/source")]
        assets: PathBuf,

        #[arg(long, env = "DIST", default_value = "./assets/dist")]
        dist: PathBuf,

        #[arg(long)]
        watch: bool,

        #[arg(long, default_value = "1000")]
        watch_debounce: u64,

        #[arg(short, long, default_value = "127.0.0.1:3001")]
        address: SocketAddr,
    },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    match args {
        Args::Id { n } => {
            for _ in 0..n {
                println!("{}", AssetId::generate());
            }
        }
        Args::Build {
            assets,
            dist,
            clean,
        } => {
            process(&assets, &dist, clean).await?;
        }
        #[cfg(feature = "server")]
        Args::Serve {
            assets,
            dist,
            watch,
            watch_debounce,
            address,
        } => {
            crate::server::server(assets, dist, watch, watch_debounce, address).await?;
        }
    }

    Ok(())
}

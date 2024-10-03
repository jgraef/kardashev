use std::path::PathBuf;

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
        Args::Build { assets, dist } => {
            process(&assets, &dist)?;
        }
    }

    Ok(())
}

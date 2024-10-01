#![allow(dead_code)]

mod atlas;
mod processor;
mod source;

use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::Error;
use uuid::Uuid;

use crate::processor::Processor;

#[derive(Debug, Parser)]
pub enum Args {
    Id {
        #[arg(short, default_value = "1")]
        n: usize,
    },
    Build {
        #[arg(long, env = "ASSETS", default_value = "./assets")]
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
                println!("{}", Uuid::new_v4());
            }
        }
        Args::Build { assets, dist } => {
            let mut processor = Processor::new(&dist);
            processor.process_directory(&assets)?;
            processor.finalize()?;
        }
    }

    Ok(())
}

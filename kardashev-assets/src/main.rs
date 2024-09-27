mod atlas;
mod processor;
mod source;

use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::Error;

use crate::processor::Processor;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long, env = "ASSETS", default_value = "./assets")]
    assets: PathBuf,

    #[arg(long, env = "DIST", default_value = "./assets/dist")]
    dist: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let mut processor = Processor::new(&args.dist);
    processor.process_directory(&args.assets)?;
    processor.finalize()?;

    Ok(())
}

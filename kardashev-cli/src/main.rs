mod admin;
mod build;
mod serve;
mod util;

use clap::Parser;
use color_eyre::eyre::Error;

#[derive(Debug, Parser)]
pub enum Args {
    Admin(crate::admin::Args),
    Build(crate::build::Args),
    Serve(crate::serve::Args),
}

impl Args {
    pub async fn run(self) -> Result<(), Error> {
        match self {
            Self::Admin(args) => args.run().await?,
            Self::Build(args) => args.run().await?,
            Self::Serve(args) => args.run().await?,
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

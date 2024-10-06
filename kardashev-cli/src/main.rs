mod admin;
mod build;
mod serve;
mod util;

use clap::{
    builder::styling,
    Parser,
};
use color_eyre::eyre::Error;
use tracing_subscriber::EnvFilter;

const STYLES: styling::Styles = styling::Styles::styled()
    .header(styling::AnsiColor::Green.on_default().bold())
    .usage(styling::AnsiColor::Green.on_default().bold())
    .literal(styling::AnsiColor::Blue.on_default().bold())
    .placeholder(styling::AnsiColor::Cyan.on_default());

/// Kardashev command line interface
///
/// `kardashev-cli` can be used to send administrative commands to the server,
/// build assets and UI and run the server.
#[derive(Debug, Parser)]
#[command(version = clap::crate_version!(), styles = STYLES)]
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
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .init();

    let args = Args::parse();
    args.run().await?;

    Ok(())
}

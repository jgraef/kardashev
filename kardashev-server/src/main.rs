mod api;
mod error;
mod server;
mod util;

use std::net::SocketAddr;

use clap::Parser;
use server::Server;
use sqlx::PgPool;
use tracing_subscriber::EnvFilter;

use crate::{
    server::Config,
    util::graceful_shutdown,
};

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,

    #[arg(short, long, default_value = "127.0.0.1:3000")]
    bind: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<(), color_eyre::eyre::Error> {
    dotenvy::dotenv().ok();
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .init();

    let args = Args::parse();

    let db = PgPool::connect(&args.database_url).await?;

    let config = Config::cargo();

    let server = Server::new(db, config).await?;
    graceful_shutdown(server.shutdown.clone());
    server.bind(args.bind).await?;

    Ok(())
}

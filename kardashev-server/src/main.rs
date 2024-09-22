mod api;
mod error;
mod server;
mod util;

use std::net::SocketAddr;

use clap::Parser;
use server::Server;
use sqlx::PgPool;

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
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let db = PgPool::connect(&args.database_url).await?;
    let server = Server::new(db).await?;
    server.bind(args.bind).await?;

    Ok(())
}

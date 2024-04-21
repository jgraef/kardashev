mod api;
mod error;
mod server;

use std::net::SocketAddr;

use server::Server;
use sqlx::PgPool;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(long, env = "DATABASE_URL")]
    database_url: String,

    #[structopt(short, long, default_value = "127.0.0.1:3000")]
    bind: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<(), color_eyre::eyre::Error> {
    dotenvy::dotenv().ok();
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let args = Args::from_args();

    let db = PgPool::connect(&args.database_url).await?;
    let server = Server::new(db).await?;
    server.bind(args.bind).await?;

    Ok(())
}

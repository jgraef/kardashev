mod constants;
mod galaxy;
mod time_scale;

use clap::Parser;
pub use color_eyre::eyre::Error;

use crate::time_scale::time_scale;

#[derive(Debug, Parser)]
enum Args {
    TimeScale { time_per_year: Vec<String> },
}

fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    match args {
        Args::TimeScale { time_per_year } => time_scale(time_per_year)?,
    }

    Ok(())
}

mod constants;
mod galaxy;
mod time_scale;

pub use color_eyre::eyre::Error;
use structopt::StructOpt;

use crate::time_scale::time_scale;

#[derive(Debug, StructOpt)]
enum Args {
    TimeScale {
        time_per_year: Vec<String>,
    },
    Galaxy {
        #[structopt(long)]
        num_stars: Option<u64>,

        #[structopt(long)]
        diameter: Option<f32>,
    },
}

fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let args = Args::from_args();
    match args {
        Args::TimeScale { time_per_year } => time_scale(time_per_year)?,
        Args::Galaxy {
            num_stars,
            diameter,
        } => todo!(),
    }

    Ok(())
}

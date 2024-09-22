#![feature(btree_cursors)]

mod gaia;
mod render;
mod utils;

use std::path::{
    Path,
    PathBuf,
};

use color_eyre::eyre::Error;
use gaia::Data;
use indicatif::{
    ProgressBar,
    ProgressStyle,
};
use nalgebra::{
    Point3,
    Rotation3,
    Vector3,
};
use sqlx::SqlitePool;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(long, env = "DATABASE_URL")]
    database_url: String,

    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    Render {
        #[structopt(short, long)]
        output: PathBuf,
        path: PathBuf,
        #[structopt(short, long, default_value = "top-down")]
        view: render::View,
        #[structopt(short, long, default_value = "1024")]
        width: u32,
    },
    Export {
        #[structopt(short, long)]
        output: PathBuf,
        path: PathBuf,
        #[structopt(short, long, default_value = "1024")]
        limit_per_file: u64,
    },
    Import {
        path: PathBuf,
        database: PathBuf,
        threads: Option<usize>,
        buf_size: Option<usize>,
    },
}

impl Args {
    async fn run(self) -> Result<(), Error> {
        //let mut db = PgPool::connect(&self.database_url).await?;

        match self.command {
            Command::Render {
                output,
                path,
                view,
                width,
            } => {
                render::render(output, path, view, width).await?;
            }
            Command::Export {
                output,
                path,
                limit_per_file,
            } => {
                render::export(output, path, limit_per_file).await?;
            }
            Command::Import {
                path,
                database,
                threads,
                buf_size,
            } => {
                //import(path, database, threads, buf_size).await?;
                todo!();
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let args = Args::from_args();
    args.run().await?;

    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i8)]
enum StarType {
    Single = 1,
    Binary = 2,
    WhiteDwarf = 3,
}

struct Record {
    source_id: u64,
    position: Point3<f64>,
    t_eff: f32,
    luminosity: f32,
    radius: f32,
    mass: f32,
    age: f32,
    ty: StarType,
}

impl Record {
    fn from_gaia(record: gaia::Record) -> Option<Self> {
        let source = record.gaia_source;
        let astro = record.astrophysical_parameters?;

        let props = [
            (astro.classprob_dsc_combmod_star, StarType::Single),
            (astro.classprob_dsc_combmod_binarystar, StarType::Binary),
            (astro.classprob_dsc_combmod_whitedwarf, StarType::WhiteDwarf),
        ];
        let Some((_, ty)) = props
            .into_iter()
            .filter_map(|(prop, ty)| Some((prop?, ty)))
            .max_by(|(p1, _), (p2, _)| p1.partial_cmp(p2).unwrap())
        else {
            return None;
        };

        let longitude = source.l?.to_radians();
        let latitude = source.b?.to_radians();
        let rotation = Rotation3::from_axis_angle(&Vector3::z_axis(), longitude)
            * Rotation3::from_axis_angle(&Vector3::x_axis(), latitude);
        let distance = astro.distance_gspphot? as f64;
        let position = Point3::from(rotation * (distance * *Vector3::y_axis()));

        let t_eff = source.teff_gspphot?;
        let luminosity = astro.lum_flame?;
        let radius = astro.radius_flame?;
        let mass = astro.mass_flame?;
        let age = astro.age_flame?;

        Some(Self {
            source_id: source.source_id,
            position,
            t_eff,
            luminosity,
            radius,
            mass,
            age,
            ty,
        })
    }
}

/*
async fn import(
    path: impl AsRef<Path>,
    database: impl AsRef<Path>,
    threads: Option<usize>,
    buf_size: Option<usize>,
) -> Result<(), Error> {
    let database = SqlitePool::connect(&format!("sqlite:{}", database.as_ref().display())).await?;
    sqlx::migrate!("./migrations").run(&database).await?;

    let data = Data::open(path).await?;
    let mut records = data.parallel(threads, buf_size);

    let (_, total) = records.progress();
    let progress_bar = ProgressBar::new(total as _);
    progress_bar.set_style(
        ProgressStyle::with_template(
            "[{pos}/{len}] {spinner:.green} {wide_bar:.cyan/blue} ({eta})",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    while let Some(record) = records.read_record().await? {
        if let Some(record) = Record::from_gaia(record) {
            let source_id = record.source_id as i64;
            let ty = record.ty as i8;
            sqlx::query!(
                r#"
                    INSERT INTO stars (
                        source_id,
                        pos_x,
                        pos_y,
                        pos_z,
                        t_eff,
                        lum,
                        radius,
                        mass,
                        age,
                        type
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                source_id,
                record.position.x,
                record.position.y,
                record.position.z,
                record.t_eff,
                record.luminosity,
                record.radius,
                record.mass,
                record.age,
                ty,
            )
            .execute(&database)
            .await?;
        }

        let (progress, _) = records.progress();
        progress_bar.set_position(progress as _);
    }

    Ok(())
}
*/

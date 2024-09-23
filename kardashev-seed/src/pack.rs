use std::{
    future::Future,
    path::Path,
    time::Instant,
};

use futures::{
    pin_mut,
    FutureExt,
};
use indicatif::{
    ProgressBar,
    ProgressStyle,
};
use nalgebra::{
    Point3,
    Rotation3,
    Vector3,
};
use sqlx::sqlite::SqliteConnectOptions;
use uuid::Uuid;

use crate::{
    config::Config,
    gaia::{self, HealPixRange},
    Error,
};

pub async fn pack(
    config: Config,
    output: impl AsRef<Path>,
    gaia_path: impl AsRef<Path>,
) -> Result<(), Error> {
    let db = sqlx::SqlitePool::connect_with(
        SqliteConnectOptions::new()
            .filename(output)
            .create_if_missing(true),
    )
    .await?;
    sqlx::migrate!("./migrations").run(&db).await?;

    let data = gaia::Data::open(gaia_path).await?;
    let mut records = data.parallel(None, None);
    //let mut records = data.sequential();

    let (_, num_partitions) = records.progress();
    let progress_bar = ProgressBar::new(num_partitions as _);
    progress_bar.set_style(
        ProgressStyle::with_template(
            "[{pos}/{len}] {spinner:.green} {wide_bar:.cyan/blue} ({eta})",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    async fn next_record(
        records: &mut gaia::ParallelReader,
        abort_signal: impl Future<Output = ()> + Unpin,
    ) -> Result<Option<gaia::Record>, Error> {
        tokio::select! {
            _ = abort_signal => Ok(None),
            result = records.read_record() => result
        }
    }

    let ctrl_c = tokio::signal::ctrl_c().map(|_| ());
    pin_mut!(ctrl_c);

    let t_start = Instant::now();

    while let Some(record) = next_record(&mut records, &mut ctrl_c).await? {
        struct Record {
            healpix_range: HealPixRange,
            source_id: u64,
            position: Point3<f64>,
            t_eff: f32,
            absolute_magnitude: f32,
            luminosity: f32,
            radius: f32,
            mass: f32,
            age: f32,
        }

        fn convert_record(record: &gaia::Record, config: &Config) -> Option<Record> {
            let astro = record.astrophysical_parameters.as_ref()?;

            let distance = astro.distance_msc?;
            if config.local_bubble.map_or(false, |d| d < distance) {
                return None;
            }

            let position = {
                let longitude = record.gaia_source.l?;
                let latitude = record.gaia_source.b?;
                let rotation = Rotation3::from_axis_angle(&Vector3::z_axis(), longitude)
                    * Rotation3::from_axis_angle(&Vector3::x_axis(), latitude);
                Point3::from(rotation * (distance as f64 * *Vector3::y_axis()))
            };

            let absolute_magnitude = astro.mg_gspphot?;
            let luminosity = astro.lum_flame?;
            let radius = astro.radius_gspphot?;
            let mass = astro.mass_flame?;
            let age = astro.age_flame?;

            Some(Record {
                healpix_range: record.healpix_range,
                source_id: record.gaia_source.source_id,
                position,
                t_eff: record.gaia_source.teff_gspphot?,
                absolute_magnitude,
                luminosity,
                radius,
                mass,
                age,
            })
        }

        if let Some(record) = convert_record(&record, &config) {
            let id = Uuid::new_v4().to_string();
            let source_id = record.source_id as i64;
            sqlx::query!(
                r#"
                INSERT INTO star (
                    id,
                    position_x,
                    position_y,
                    position_z,
                    t_eff,
                    absolute_magnitude,
                    luminosity,
                    radius,
                    mass,
                    age,
                    healpix_start,
                    healpix_end,
                    source_id
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                id,
                record.position.x,
                record.position.y,
                record.position.z,
                record.t_eff,
                record.absolute_magnitude,
                record.luminosity,
                record.radius,
                record.mass,
                record.age,
                record.healpix_range.start,
                record.healpix_range.end,
                source_id,
            )
            .execute(&db)
            .await?;
        }

        let (progress, _) = records.progress();
        progress_bar.set_position(progress as _);
    }

    let time = t_start.elapsed();
    tracing::info!("packing took {} s", time.as_secs());

    Ok(())
}

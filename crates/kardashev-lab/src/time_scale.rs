#![allow(dead_code)]

use std::{
    borrow::Cow,
    f64::consts::PI,
    time::Duration,
};

use tabled::{
    settings::Style,
    Table,
    Tabled,
};

use crate::{
    constants::{
        GALAXY_DIAMETER,
        PROXIMA_CENTAURI_DINSTANCE,
        STARS_IN_MILKY_WAY,
        SUN_GALACTIC_PERIOD,
    },
    Error,
};

pub fn time_scale(time_per_year: Vec<String>) -> Result<(), Error> {
    struct Row {
        duration_per_year: Duration, // time per in-game year
        sun_revolution: Duration,
        galaxy_crossing: Duration,  // with 0.5c
        proxima_centauri: Duration, // with 0.5c
        star_systems_24h: u64,      // with 0.5c
        hundred_k_years: Duration,
    }

    impl Row {
        pub fn from_duration_per_year(duration_per_year: Duration) -> Self {
            let time_scale = duration_per_year.as_secs_f64();
            Self {
                duration_per_year,
                sun_revolution: Duration::from_secs_f64(SUN_GALACTIC_PERIOD * time_scale),
                galaxy_crossing: Duration::from_secs_f64(GALAXY_DIAMETER * 2.0 * time_scale),
                proxima_centauri: Duration::from_secs_f64(
                    PROXIMA_CENTAURI_DINSTANCE * 2.0 * time_scale,
                ),
                star_systems_24h: (STARS_IN_MILKY_WAY as f64 / (GALAXY_DIAMETER.powi(2) * PI)
                    * (24.0 * 3600.0 / time_scale * 2.0).powi(2)
                    * PI) as u64,
                hundred_k_years: Duration::from_secs_f64(100_000.0 * time_scale),
            }
        }
    }

    impl Tabled for Row {
        const LENGTH: usize = 6;

        fn fields(&self) -> Vec<Cow<'_, str>> {
            vec![
                humantime::format_duration(self.duration_per_year)
                    .to_string()
                    .into(),
                //humantime::format_duration(self.sun_revolution).to_string().into(),
                humantime::format_duration(self.galaxy_crossing)
                    .to_string()
                    .into(),
                humantime::format_duration(self.proxima_centauri)
                    .to_string()
                    .into(),
                self.star_systems_24h.to_string().into(),
                humantime::format_duration(self.hundred_k_years)
                    .to_string()
                    .into(),
            ]
        }

        fn headers() -> Vec<Cow<'static, str>> {
            vec![
                "Time scale".into(),
                //"Sun revolution".into(),
                "Galaxy crossing".into(),
                "Proxima Centuri".into(),
                "Star systems in 24h".into(),
                "100k years".into(),
            ]
        }
    }

    let mut time_per_year = time_per_year
        .iter()
        .map(|t| humantime::parse_duration(t))
        .collect::<Result<Vec<Duration>, _>>()?;

    if time_per_year.is_empty() {
        time_per_year.push(Duration::from_secs(1));
        time_per_year.push(Duration::from_secs(3 * 60));
        time_per_year.push(Duration::from_secs(5 * 60));
        time_per_year.push(Duration::from_secs(10 * 60));
        time_per_year.push(Duration::from_secs(15 * 60));
        time_per_year.push(Duration::from_secs(30 * 60));
        time_per_year.push(Duration::from_secs(60 * 60));
    }

    let rows = time_per_year
        .into_iter()
        .map(|t| Row::from_duration_per_year(t))
        .collect::<Vec<Row>>();

    let mut table = Table::new(&rows);
    table.with(Style::markdown());
    println!("{table}");

    Ok(())
}

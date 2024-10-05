use std::path::Path;

use indicatif::{
    ProgressBar,
    ProgressStyle,
};
use itertools::Itertools;
use kardashev_client::ApiClient;
use kardashev_protocol::{
    admin::CreateStar,
    model::star::CatalogIds,
};
use nalgebra::Point3;

use crate::admin::{
    catalog::hyg::{
        self,
        Record,
    },
    utils::teff_color::teff_color,
    Error,
};

pub async fn import_stars(
    api: &ApiClient,
    path: impl AsRef<Path>,
    batch_size: usize,
    num_closest: Option<usize>,
) -> Result<(), Error> {
    let reader = hyg::Reader::open(path)?;

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg}")
            .unwrap()
            .tick_strings(&["-", "\\", "|", "/"]),
    );
    pb.set_message("reading stars...");

    let mut stars = reader.collect::<Result<Vec<Record>, Error>>()?;
    if let Some(num_closest) = num_closest {
        if num_closest < stars.len() {
            stars.sort_by(|a, b| a.dist.partial_cmp(&b.dist).unwrap());
            stars.resize_with(num_closest, || unreachable!());
        }
    }

    let chunks = stars.into_iter().chunks(batch_size);
    for chunk in &chunks {
        let mut batch = vec![];

        for record in chunk {
            let Some(spect) = record.spect
            else {
                continue;
            };
            let m = approximate_mass(record.lum);
            let r = approximate_radius(m);
            let t_eff = approximate_teff(record.lum, r);
            let color = teff_color(t_eff);

            batch.push(CreateStar {
                position: Point3::new(record.x, record.y, record.z),
                effective_temperature: t_eff,
                color: color.into(),
                absolute_magnitude: record.absmag,
                luminousity: record.lum,
                radius: r,
                mass: m,
                spectral_type: spect,
                name: record.proper.clone(),
                catalog_ids: CatalogIds {
                    hyg: Some(record.id),
                    hip: record.hip,
                    hd: record.hd,
                    hr: record.hr,
                    gl: record.gl,
                    bf: record.bf,
                },
            });

            if let Some(name) = record.proper {
                pb.set_message(name);
            }
            else {
                pb.set_message(format!("#{}", record.id))
            }
            pb.tick();
        }

        pb.set_message("uploading batch");
        pb.tick();

        api.create_stars(batch).await?;
    }

    Ok(())
}

fn approximate_mass(lum: f32) -> f32 {
    if lum < 0.033 {
        4.3 * lum.powf(0.43)
    }
    else if lum < 16. {
        lum.powf(0.25)
    }
    else if lum < 1700000. {
        0.7 * lum.powf(0.3)
    }
    else {
        0.000031 * lum
    }
}

fn approximate_radius(m: f32) -> f32 {
    if m < 1. {
        m.powf(0.8)
    }
    else {
        m.powf(0.5)
    }
}

fn approximate_teff(lum: f32, r: f32) -> f32 {
    (lum / r.powi(2)).powf(0.25) * 5778.0
}

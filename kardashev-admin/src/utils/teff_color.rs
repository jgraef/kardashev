use lazy_static::lazy_static;
use palette::LinSrgb;
use serde::Deserialize;

lazy_static! {
    static ref TEFF_COLORS: Vec<(u32, LinSrgb)> = parse_csv();
}

fn parse_csv() -> Vec<(u32, LinSrgb)> {
    #[derive(Deserialize)]
    struct Row {
        t_eff: u32,
        r: f32,
        g: f32,
        b: f32,
    }

    const CSV: &'static [u8] = include_bytes!("teff-rgb.csv");
    let mut reader = csv::Reader::from_reader(CSV);
    reader
        .deserialize::<Row>()
        .map(|row| {
            let row = row.unwrap();
            (row.t_eff, LinSrgb::new(row.r, row.g, row.b))
        })
        .collect()
}

pub fn teff_color(t_eff: f32) -> LinSrgb {
    let index = match TEFF_COLORS.binary_search_by_key(&(t_eff as u32), |(t_eff, _)| *t_eff) {
        Ok(index) | Err(index) => index,
    };

    if index + 1 == TEFF_COLORS.len() {
        return TEFF_COLORS[TEFF_COLORS.len() - 1].1;
    }

    let t_lower = TEFF_COLORS[index].0 as f32;
    let t_upper = TEFF_COLORS[index + 1].0 as f32;

    let k = (t_eff - t_lower) / (t_upper - t_lower);

    let rgb_lower = TEFF_COLORS[index].1;
    let rgb_upper = TEFF_COLORS[index + 1].1;
    LinSrgb::new(
        (1.0 - k) * rgb_lower.red + k * rgb_upper.red,
        (1.0 - k) * rgb_lower.green + k * rgb_upper.green,
        (1.0 - k) * rgb_lower.blue + k * rgb_upper.blue,
    )
}

use std::{
    collections::BTreeMap,
    ops::Bound,
};

use lazy_static::lazy_static;
use palette::LinSrgb;
use serde::Deserialize;

lazy_static! {
    static ref TEFF_COLORS: TeffColorTable =
        TeffColorTable::load_table(include_str!("teff-rgb.csv"));
}

struct TeffColorTable {
    data: BTreeMap<u32, LinSrgb>,
}

impl TeffColorTable {
    fn load_table(table: &str) -> Self {
        #[derive(Deserialize)]
        struct Row {
            t_eff: u32,
            r: f32,
            g: f32,
            b: f32,
        }

        let mut reader = csv::Reader::from_reader(table.as_bytes());
        let mut data = BTreeMap::new();

        for row in reader.deserialize::<Row>() {
            let row = row.unwrap();
            data.insert(row.t_eff, LinSrgb::new(row.r, row.g, row.b));
        }

        Self { data }
    }

    fn get(&self, t_eff: f32) -> LinSrgb {
        let t_eff_int = t_eff as u32;

        let cursor = self.data.lower_bound(Bound::Included(&t_eff_int));

        match (cursor.peek_next(), cursor.peek_prev()) {
            (Some((t_upper, rgb_upper)), Some((t_lower, rgb_lower))) => {
                let t_lower = *t_lower as f32;
                let t_upper = *t_upper as f32;

                let k = (t_eff - t_lower) / (t_upper - t_lower);

                LinSrgb::new(
                    (1.0 - k) * rgb_lower.red + k * rgb_upper.red,
                    (1.0 - k) * rgb_lower.green + k * rgb_upper.green,
                    (1.0 - k) * rgb_lower.blue + k * rgb_upper.blue,
                )
            }
            (Some((_, rgb)), None) | (None, Some((_, rgb))) => *rgb,
            _ => unreachable!(),
        }
    }
}

pub fn teff_color(t_eff: f32) -> LinSrgb {
    TEFF_COLORS.get(t_eff)
}

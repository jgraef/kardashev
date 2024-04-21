use std::{
    collections::BTreeMap,
    ops::Bound,
};

use lazy_static::lazy_static;
use palette::LinSrgb;

lazy_static! {
    pub static ref TEFF_COLORS: TeffColorTable =
        TeffColorTable::load_table(include_str!("teff-rgb.csv"));
}

pub struct TeffColorTable {
    data: BTreeMap<u32, LinSrgb>,
}

impl TeffColorTable {
    fn load_table(table: &str) -> Self {
        let mut data = BTreeMap::new();

        for line in table.lines() {
            let mut cells = line.split(',');
            let t_eff = cells.next().unwrap().parse().unwrap();
            let color = LinSrgb::new(
                cells.next().unwrap().parse().unwrap(),
                cells.next().unwrap().parse().unwrap(),
                cells.next().unwrap().parse().unwrap(),
            );
            data.insert(t_eff, color);
        }

        Self { data }
    }

    pub fn get(&self, t_eff: f32) -> Option<LinSrgb> {
        let t_eff_int = t_eff as u32;

        let cursor = self.data.lower_bound(Bound::Included(&t_eff_int));

        let (t_upper, rgb_upper) = cursor.peek_next()?;
        let (t_lower, rgb_lower) = cursor.peek_prev()?;

        let k = t_eff / ((*t_upper as f32) - (*t_lower as f32));

        let color = LinSrgb::new(
            (1.0 - k) * rgb_lower.red + k * rgb_upper.red,
            (1.0 - k) * rgb_lower.green + k * rgb_upper.green,
            (1.0 - k) * rgb_lower.blue + k * rgb_upper.blue,
        );

        Some(color)
    }
}

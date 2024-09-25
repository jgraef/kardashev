pub mod teff_color;

use std::fmt::Display;

use chrono::TimeDelta;
use serde::{
    Deserialize,
    Deserializer,
};

pub fn invalid_option<'de, D, T: Deserialize<'de>>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(T::deserialize(deserializer).ok())
}

pub fn format_uptime(td: TimeDelta) -> FormattedUptime {
    FormattedUptime(td)
}

#[derive(Debug)]
pub struct FormattedUptime(TimeDelta);

impl Display for FormattedUptime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let days = self.0.num_days();
        let hours = self.0.num_hours() % 24;
        let minutes = self.0.num_minutes() % 60;
        let seconds = self.0.num_seconds() % 60;
        if days > 0 {
            write!(f, "{days} days ")?;
        }
        write!(f, "{hours}h {minutes}m {seconds}s")
    }
}

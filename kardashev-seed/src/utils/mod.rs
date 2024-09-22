pub mod teff_color;

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

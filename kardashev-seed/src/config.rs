use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub local_bubble: Option<f64>,
}

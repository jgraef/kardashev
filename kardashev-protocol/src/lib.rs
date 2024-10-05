pub mod admin;
pub mod assets;
pub mod model;

use std::fmt::Display;

use chrono::{
    DateTime,
    Utc,
};
use semver::Version;
use serde::{
    de::DeserializeOwned,
    Deserialize,
    Serialize,
};
/// re-export for the `asset_id!` macro
pub use uuid;

use crate::model::star::Star;

pub const PROTOCOL_VERSION: Version = semver_macro::version!("0.1.0");

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerStatus {
    pub server_version: Version,
    pub up_since: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetStarsResponse {
    pub stars: Vec<Star>,
}

#[derive(Debug, thiserror::Error)]
pub struct PrettyJsonError {
    #[source]
    source: serde_json::Error,
    pretty: Option<(usize, usize, String)>,
}

impl Display for PrettyJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", &self.source)?;

        if let Some((line, col, pretty)) = &self.pretty {
            for (line_num, line_str) in pretty.lines().enumerate() {
                if line.abs_diff(line_num) < 5 {
                    writeln!(f, "{:>4} {line_str}", line_num + 1)?;
                }
                if *line == line_num {
                    struct Dashes(usize);
                    impl Display for Dashes {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            for _ in 0..self.0 {
                                write!(f, "-")?;
                            }
                            Ok(())
                        }
                    }
                    writeln!(f, "     {}^", Dashes(col.saturating_sub(1)))?;
                }
            }
        }

        Ok(())
    }
}

pub fn json_decode<T: DeserializeOwned>(json: impl AsRef<[u8]>) -> Result<T, PrettyJsonError> {
    let json = json.as_ref();
    serde_json::from_slice(json).map_err(|source| {
        if source.is_data() {
            let json: serde_json::Value = serde_json::from_slice(json).unwrap();
            let pretty_json = serde_json::to_string_pretty(&json).unwrap();
            match serde_json::from_str::<T>(&pretty_json) {
                Ok(_) => panic!("pretty printed JSON parsed successfully"),
                Err(error) => {
                    PrettyJsonError {
                        source,
                        pretty: Some((
                            error.line() - 1,
                            error.column().saturating_sub(1),
                            pretty_json,
                        )),
                    }
                }
            }
        }
        else {
            PrettyJsonError {
                source,
                pretty: None,
            }
        }
    })
}

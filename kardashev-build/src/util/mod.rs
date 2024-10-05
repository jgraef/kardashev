pub mod process;
pub mod watch;

use std::path::Path;

use chrono::{
    DateTime,
    Utc,
};
use walkdir::WalkDir;

pub fn path_modified_timestamp(
    path: impl AsRef<Path>,
    fold: impl Fn(DateTime<Utc>, DateTime<Utc>) -> DateTime<Utc>,
) -> Result<DateTime<Utc>, std::io::Error> {
    let path = path.as_ref();

    let metadata = path.metadata()?;
    let mut modified_time: DateTime<Utc> = metadata.modified()?.into();

    if metadata.is_dir() {
        for result in WalkDir::new(path) {
            let entry = result?;
            let metadata = entry.metadata()?;
            modified_time = fold(modified_time, metadata.modified()?.into());
        }
    }

    Ok(modified_time)
}

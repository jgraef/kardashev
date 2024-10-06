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
) -> Result<Option<DateTime<Utc>>, std::io::Error> {
    let path = path.as_ref();

    let mut modified_time = None;

    for result in WalkDir::new(path) {
        let entry = result?;
        let metadata = path.metadata()?;
        if metadata.is_file() {
            let file_modified_time = entry.metadata()?.modified()?.into();
            if let Some(modified_time) = &mut modified_time {
                *modified_time = fold(*modified_time, file_modified_time);
            }
            else {
                modified_time = Some(file_modified_time);
            }
        }
    }

    Ok(modified_time)
}

use std::process::Stdio;

use tokio::process::Command;

use crate::util::process::{
    ExitStatusError,
    OutputExt,
};

#[derive(Debug, thiserror::Error)]
#[error("git error")]
pub enum Error {
    Io(#[from] std::io::Error),
    ExitStatus(#[from] ExitStatusError),
    Utf8(#[from] std::str::Utf8Error),
}

#[derive(Clone, Debug, Default)]
pub struct Git;

impl Git {
    pub async fn head(&self) -> Result<String, Error> {
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("HEAD")
            .stdout(Stdio::piped())
            .spawn()?
            .wait_with_output()
            .await?
            .into_result()?;
        let commit = std::str::from_utf8(output.stdout.trim_ascii())?;
        Ok(commit.to_owned())
    }
}

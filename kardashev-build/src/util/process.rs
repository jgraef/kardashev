use std::process::{
    ExitStatus,
    Output,
};

use kardashev_protocol::{
    json_decode,
    PrettyJsonError,
};
use serde::de::DeserializeOwned;

#[derive(Debug, thiserror::Error)]
#[error("program exited with status: {exit_status}")]
pub struct ExitStatusError {
    exit_status: ExitStatus,
}

pub trait ExitStatusExt {
    fn into_result(self) -> Result<(), ExitStatusError>;
}

impl ExitStatusExt for ExitStatus {
    fn into_result(self) -> Result<(), ExitStatusError> {
        if self.success() {
            Ok(())
        }
        else {
            Err(ExitStatusError { exit_status: self })
        }
    }
}

pub trait OutputExt: Sized {
    fn into_result(self) -> Result<Self, ExitStatusError>;
}

impl OutputExt for Output {
    fn into_result(self) -> Result<Self, ExitStatusError> {
        self.status.into_result()?;
        Ok(self)
    }
}

pub trait OutputJsonExt: OutputExt {
    fn into_json_result<T: DeserializeOwned>(self) -> Result<T, OutputJsonError>;
}

impl OutputJsonExt for Output {
    fn into_json_result<T: DeserializeOwned>(self) -> Result<T, OutputJsonError> {
        let output = self.into_result()?;
        let value: T = json_decode(&output.stdout)?;
        Ok(value)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub enum OutputJsonError {
    ExitStatus(#[from] ExitStatusError),
    Json(#[from] PrettyJsonError),
}

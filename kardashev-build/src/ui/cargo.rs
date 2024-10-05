use std::{
    collections::HashMap,
    path::{
        Path,
        PathBuf,
    },
    process::Stdio,
};

use serde::{
    Deserialize,
    Serialize,
};
use tokio::process::Command;

use crate::util::process::{
    ExitStatusError,
    ExitStatusExt,
    OutputJsonError,
    OutputJsonExt,
};

#[derive(Debug, thiserror::Error)]
#[error("cargo error")]
pub enum Error {
    Io(#[from] std::io::Error),
    ExitStatus(#[from] ExitStatusError),
    Json(#[from] kardashev_protocol::PrettyJsonError),
}

impl From<OutputJsonError> for Error {
    fn from(error: OutputJsonError) -> Self {
        match error {
            OutputJsonError::ExitStatus(exit_status_error) => Self::ExitStatus(exit_status_error),
            OutputJsonError::Json(pretty_json_error) => Self::Json(pretty_json_error),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Cargo {
    crate_path: PathBuf,
    cargo_path: PathBuf,
}

impl Cargo {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            crate_path: path.as_ref().to_owned(),
            cargo_path: PathBuf::from("cargo"),
        }
    }

    pub fn with_cargo_path(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.cargo_path = path.as_ref().to_owned();
        self
    }

    fn command(&self) -> Command {
        let mut command = Command::new(&self.cargo_path);
        command.current_dir(&self.crate_path);
        command
    }

    pub async fn locate_workspace(&self) -> Result<PathBuf, Error> {
        #[derive(Deserialize)]
        struct Output {
            root: PathBuf,
        }
        let output: Output = self
            .command()
            .arg("locate-project")
            .arg("--workspace")
            .stdout(Stdio::piped())
            .spawn()?
            .wait_with_output()
            .await?
            .into_json_result()?;
        Ok(output.root)
    }

    pub async fn manifest(&self) -> Result<Manifest, Error> {
        Ok(self
            .command()
            .arg("read-manifest")
            .stdout(Stdio::piped())
            .spawn()?
            .wait_with_output()
            .await?
            .into_json_result()?)
    }

    pub async fn build(&self, target: Option<&str>) -> Result<(), Error> {
        let mut command = self.command();
        command.arg("build");
        if let Some(target) = target {
            command.arg("target");
            command.arg(target);
        }
        command.spawn()?.wait().await?.into_result()?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub id: String,
    pub license: Option<String>,
    pub license_file: Option<String>,
    pub description: Option<String>,
    pub source: Option<String>,
    pub dependencies: Vec<Dependency>,
    pub targets: Vec<Target>,
    pub features: HashMap<String, String>,
    pub manifest_path: PathBuf,
    pub metadata: HashMap<String, serde_json::Value>,
    pub publish: Option<bool>,
    pub authors: Vec<String>,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
    pub readme: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub documentation: Option<String>,
    pub edition: Option<String>,
    // todo
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub source: Option<String>,
    pub req: String,
    pub kind: Option<String>,
    pub rename: Option<String>,
    pub optional: bool,
    pub use_default_features: Option<bool>,
    pub features: Vec<String>,
    pub target: Option<String>,
    pub registry: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Target {
    pub kind: Vec<String>,
    pub crate_types: Vec<String>,
    pub name: String,
    pub src_path: String,
    pub edition: String,
    pub doc: bool,
    pub doctest: bool,
    pub test: bool,
}

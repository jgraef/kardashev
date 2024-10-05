use std::{
    collections::HashMap,
    path::{
        Path,
        PathBuf,
    },
    process::{
        ExitStatus,
        Stdio,
    },
};

use kardashev_protocol::json_decode;
use serde::{
    Deserialize,
    Serialize,
};
use tokio::process::Command;

#[derive(Debug, thiserror::Error)]
#[error("cargo error")]
pub enum Error {
    Io(#[from] std::io::Error),
    ExitStatus { exit_status: ExitStatus },
    Json(#[from] kardashev_protocol::PrettyJsonError),
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
        let output = self
            .command()
            .arg("locate-project")
            .arg("--workspace")
            .stdout(Stdio::piped())
            .spawn()?
            .wait_with_output()
            .await?;
        exit_status_into_result(output.status)?;
        #[derive(Deserialize)]
        struct Output {
            root: PathBuf,
        }
        let output: Output = json_decode(&output.stdout)?;
        Ok(output.root)
    }

    pub async fn manifest(&self) -> Result<Manifest, Error> {
        let output = self
            .command()
            .arg("read-manifest")
            .stdout(Stdio::piped())
            .spawn()?
            .wait_with_output()
            .await?;
        exit_status_into_result(output.status)?;
        let manifest: Manifest = json_decode(&output.stdout)?;
        Ok(manifest)
    }

    pub async fn build(&self, target: Option<&str>) -> Result<(), Error> {
        let mut command = self.command();
        command.arg("build");
        if let Some(target) = target {
            command.arg("target");
            command.arg(target);
        }
        let exit_status = command.spawn()?.wait().await?;
        exit_status_into_result(exit_status)?;
        Ok(())
    }
}

fn exit_status_into_result(exit_status: ExitStatus) -> Result<(), Error> {
    if exit_status.success() {
        Ok(())
    }
    else {
        Err(Error::ExitStatus { exit_status })
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

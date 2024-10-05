use std::{
    fmt::Debug,
    path::Path,
};

use tokio::process::Command;

use crate::util::process::{
    ExitStatusError,
    ExitStatusExt,
};

pub async fn wasm_bindgen(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    output_name: &str,
) -> Result<(), WasmBindgenError> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();

    #[cfg(feature = "wasm-bindgen-lib")]
    {
        wasm_bindgen_lib(input_path, output_path, output_name).await?;
    }

    #[cfg(not(feature = "wasm-bindgen-lib"))]
    {
        if let Err(error) = wasm_bindgen_bin_test().await {
            tracing::error!(?error, "wasm-bindgen binary failed");
            tracing::error!("You either need to install wasm-bindgen (`cargo install wasm-bindgen-cli`), or enable the `wasm-bindgen-lib` feature.");
            return Err(WasmBindgenError::NoBackend);
        }
        else {
            wasm_bindgen_bin(input_path, output_path, output_name).await?;
        }
    }

    Ok(())
}

#[cfg(feature = "wasm-bindgen-lib")]
async fn wasm_bindgen_lib(
    input_path: &Path,
    output_dir: &Path,
    output_name: &str,
) -> Result<(), WasmBindgenLibError> {
    let mut bindgen = wasm_bindgen_cli_support::Bindgen::new();
    bindgen.input_path(&input_path).web(true).unwrap();
    bindgen.out_name(&output_name);

    let output_dir = output_dir.to_owned();
    tokio::task::spawn_blocking(move || bindgen.generate(output_dir))
        .await
        .unwrap()
        .map_err(WasmBindgenLibError::new)?;

    Ok(())
}

#[allow(dead_code)]
async fn wasm_bindgen_bin(
    input_path: &Path,
    output_dir: &Path,
    output_name: &str,
) -> Result<(), WasmBindgenBinError> {
    Command::new("wasm-bindgen")
        .arg("--out-dir")
        .arg(output_dir)
        .arg("--out-name")
        .arg(output_name)
        .arg("--target")
        .arg("web")
        .arg("--no-typescript")
        .arg(input_path)
        .spawn()?
        .wait()
        .await?
        .into_result()?;
    Ok(())
}

#[allow(dead_code)]
async fn wasm_bindgen_bin_test() -> Result<(), WasmBindgenBinError> {
    Command::new("wasm-bindgen")
        .arg("--version")
        .spawn()?
        .wait()
        .await?
        .into_result()?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
#[error("wasm-bindgen error")]
pub enum WasmBindgenError {
    #[cfg(feature = "wasm-bindgen-lib")]
    Lib(#[from] WasmBindgenLibError),
    Bin(#[from] WasmBindgenBinError),
    #[error("no wasm-bindgen backend")]
    NoBackend,
}

#[cfg(feature = "wasm-bindgen-lib")]
#[derive(Debug, thiserror::Error)]
#[error("wasm-bindgen error: {message}")]
pub struct WasmBindgenLibError {
    message: String,
}

#[cfg(feature = "wasm-bindgen-lib")]
impl WasmBindgenLibError {
    fn new(message: impl Display) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("wasm-bindgen error")]
pub enum WasmBindgenBinError {
    Io(#[from] std::io::Error),
    ExitStatus(#[from] ExitStatusError),
}

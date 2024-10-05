mod cargo;
mod wasm_bindgen;

use std::path::Path;

use crate::{
    ui::{
        cargo::Cargo,
        wasm_bindgen::wasm_bindgen,
    },
    util::path_modified_timestamp,
};

#[derive(Debug, thiserror::Error)]
#[error("ui build error")]
pub enum Error {
    Io(#[from] std::io::Error),
    Cargo(#[from] crate::ui::cargo::Error),
    WasmBindgen(#[from] crate::ui::wasm_bindgen::WasmBindgenError),
}

pub async fn compile_ui(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<(), Error> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();
    tracing::info!(input = %input_path.display(), output = %output_path.display(), "compiling ui");

    std::fs::create_dir_all(&output_path)?;

    // check freshness
    let input_modified_time = path_modified_timestamp(input_path, std::cmp::max)?;
    let output_modified_time = path_modified_timestamp(output_path, std::cmp::min)?;
    if input_modified_time <= output_modified_time {
        tracing::debug!("not modified since last build. skipping.");
        return Ok(());
    }

    let cargo = Cargo::new(&input_path);

    let manifest = cargo.manifest().await?;
    if manifest.targets.len() != 1 {
        // todo: don't panic
        panic!("Unexpected number of targets: {}", manifest.targets.len());
    }

    let target_name = &manifest.targets[0].name;
    tracing::debug!(%target_name);

    let workspace_path = cargo.locate_workspace().await?;
    let workspace_path = workspace_path.parent().unwrap();
    tracing::debug!(workspace_path = %workspace_path.display());

    let target_path = workspace_path
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("debug")
        .join(format!("{target_name}.wasm"));
    tracing::debug!(target_path = %target_path.display());

    wasm_bindgen(&target_path, output_path, &target_name).await?;

    Ok(())
}

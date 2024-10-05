use std::{
    collections::HashMap,
    fmt::Display,
};

use kardashev_protocol::assets::AssetId;
use wasm_bindgen_cli_support::Bindgen;

use crate::{
    cargo::Cargo,
    dist,
    processor::ProcessContext,
    source::{
        Manifest,
        Wasm,
    },
    Asset,
    Error,
};

impl Asset for Wasm {
    fn register_dist_type(dist_asset_types: &mut dist::AssetTypes) {
        dist_asset_types.register::<dist::Wasm>();
    }

    fn get_assets(manifest: &Manifest) -> &HashMap<AssetId, Self> {
        &manifest.wasm
    }

    async fn process<'a, 'b: 'a>(
        &'a self,
        id: AssetId,
        context: &'a mut ProcessContext<'b>,
    ) -> Result<(), Error> {
        if !context.processing(id) {
            return Ok(());
        }

        let source_path = context.input_path(&self.source);

        // check freshness
        if context.source_path(id, &source_path)?.is_fresh() {
            tracing::debug!(%id, "not modified since last build. skipping.");
            return Ok(());
        }

        let cargo = Cargo::new(&source_path);

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

        let mut bindgen = Bindgen::new();
        bindgen.input_path(&target_path).web(true).unwrap();

        let mut output = tokio::task::spawn_blocking(move || bindgen.generate_output())
            .await
            .unwrap()
            .map_err(|e| WasmBindgenError::new(e))?;

        let js_filename = format!("{id}.js");
        let path = context.dist_path.join(&js_filename);
        std::fs::write(&path, output.js())?;

        let wasm_filename = format!("{id}.wasm");
        let path = context.dist_path.join(&wasm_filename);
        output
            .wasm_mut()
            .emit_wasm_file(&path)
            .map_err(|e| WalrusError::new(e))?;

        context.dist_assets.insert(dist::Wasm {
            id,
            label: self.label.clone(),
            js: js_filename,
            wasm: wasm_filename,
        });

        context.set_build_time(id);

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("wasm-bindgen error: {message}")]
pub struct WasmBindgenError {
    message: String,
}

impl WasmBindgenError {
    fn new(message: impl Display) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("wasm-bindgen error: {message}")]
pub struct WalrusError {
    message: String,
}

impl WalrusError {
    fn new(message: impl Display) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

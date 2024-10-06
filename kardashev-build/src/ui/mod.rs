mod cargo;
mod wasm_bindgen;

use std::{
    fs::File,
    io::{
        BufReader,
        BufWriter,
        Read,
    },
    path::Path,
};

use askama::Template;

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

#[tracing::instrument(skip_all)]
pub async fn compile_ui(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<(), Error> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();

    std::fs::create_dir_all(&output_path)?;

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

    let target_wasm_path = workspace_path
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("debug")
        .join(format!("{target_name}.wasm"));
    tracing::debug!(target_wasm_path = %target_wasm_path.display());

    let wasm_filename = format!("{target_name}_bg.wasm");
    let js_filename = format!("{target_name}.js");
    let css_filename = format!("{target_name}.css");
    let index_filename = "index.html";

    // check if all files exist
    if !output_path.join(&wasm_filename).exists()
        || !output_path.join(&js_filename).exists()
        || !output_path.join(&css_filename).exists()
        || !output_path.join(&index_filename).exists()
    {
        tracing::warn!("input file missing. rebuilding.");
    }
    else {
        // check freshness
        let input_modified_time = path_modified_timestamp(input_path, std::cmp::max)?;
        let output_modified_time = path_modified_timestamp(output_path, std::cmp::min)?;
        tracing::debug!(?input_modified_time, ?output_modified_time);
        let fresh = match (input_modified_time, output_modified_time) {
            (None, _) => true,
            (Some(input_modified_time), Some(output_modified_time))
                if input_modified_time <= output_modified_time =>
            {
                true
            }
            _ => false,
        };
        if fresh {
            tracing::debug!("not modified since last build. skipping.");
            return Ok(());
        }
    }

    tracing::info!(target = %target_name, "running `cargo build`");
    cargo.build(Some("wasm32-unknown-unknown")).await?;

    tracing::info!(target = %target_name, "running `wasm-bindgen`");
    wasm_bindgen(&target_wasm_path, output_path, &target_name).await?;

    tracing::info!("collecting CSS");
    let css_path = workspace_path
        .join("target")
        .join("css")
        .join("kardashev-ui");
    let mut css_buf = vec![];
    for result in std::fs::read_dir(&css_path)? {
        let entry = result?;
        let mut reader = BufReader::new(File::open(&entry.path())?);
        reader.read_to_end(&mut css_buf)?;
    }
    let css_output_path = output_path.join(&css_filename);
    tracing::debug!(path = %css_output_path.display(), "writing CSS file");
    std::fs::write(&css_output_path, &css_buf)?;

    tracing::debug!(target = %target_name, "generating `index.html`");
    let mut writer = BufWriter::new(File::create(output_path.join(&index_filename))?);
    IndexHtml {
        js: &js_filename,
        wasm: &wasm_filename,
        css: &css_filename,
    }
    .write_into(&mut writer)?;

    tracing::info!("done");

    Ok(())
}

#[derive(Debug, Template)]
#[template(path = "index.html")]
struct IndexHtml<'a> {
    js: &'a str,
    wasm: &'a str,
    css: &'a str,
}

use std::{
    collections::HashMap,
    fs::File,
    io::BufWriter,
};

use kardashev_protocol::assets::AssetId;

use crate::assets::{
    dist,
    processor::ProcessContext,
    source::{
        Manifest,
        Shader,
    },
    Asset,
    Error,
};

impl Asset for Shader {
    fn register_dist_type(dist_asset_types: &mut dist::AssetTypes) {
        dist_asset_types.register::<dist::Shader>();
    }

    fn get_assets(manifest: &Manifest) -> &HashMap<AssetId, Self> {
        &manifest.shaders
    }

    async fn process<'a, 'b: 'a>(
        &'a self,
        id: AssetId,
        context: &'a mut ProcessContext<'b>,
    ) -> Result<(), Error> {
        if !context.processing(id) {
            return Ok(());
        }

        let path = context.input_path(&self.path);

        if context.source_path(id, &path)?.is_fresh() {
            tracing::debug!(%id, "not modified since last build. skipping.");
            return Ok(());
        }

        let source = std::fs::read_to_string(&path)?;
        let module = naga::front::wgsl::parse_str(&source)?;
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );

        match validator.validate(&module) {
            Ok(module_info) => {
                let compiled = dist::CompiledShader {
                    label: self.label.clone(),
                    module,
                    module_info,
                };
                let filename = format!("{id}.naga");
                let path = context.dist_path.join(&filename);
                let mut writer = BufWriter::new(File::create(&path)?);
                //serde_json::to_writer_pretty(writer, &compiled)?;
                rmp_serde::encode::write(&mut writer, &compiled)?;

                context.dist_assets.insert(dist::Shader {
                    id,
                    label: self.label.clone(),
                    build_time: context.build_time,
                    naga_ir: filename,
                });

                context.set_build_time(id);
            }
            Err(error) => {
                error.emit_to_stderr_with_path(&source, &path.to_string_lossy());
            }
        }

        Ok(())
    }
}

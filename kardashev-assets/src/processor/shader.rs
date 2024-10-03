use std::{
    fs::File,
    io::BufWriter,
    path::Path,
};

use kardashev_protocol::assets::AssetId;

use crate::{
    dist,
    processor::{
        input_path,
        Process,
        Processor,
    },
    source::Shader,
    Error,
};

impl Process for Shader {
    fn process(
        &self,
        id: AssetId,
        processor: &mut Processor,
        manifest_path: &Path,
    ) -> Result<(), Error> {
        let path = input_path(manifest_path, &self.path);
        tracing::debug!(%id, path=%path.display(), "processing shader");

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
                let path = processor.dist_path.join(&filename);
                let mut writer = BufWriter::new(File::create(&path)?);
                //serde_json::to_writer_pretty(writer, &compiled)?;
                rmp_serde::encode::write(&mut writer, &compiled)?;

                processor.dist_manifest.shaders.push(dist::Shader {
                    id,
                    label: self.label.clone(),
                    naga_ir: filename,
                });
            }
            Err(error) => {
                error.emit_to_stderr_with_path(&source, &path.to_string_lossy());
            }
        }

        Ok(())
    }
}

use std::{
    fs::File,
    io::BufReader,
    path::Path,
};

use kardashev_protocol::assets::{
    AssetId,
    MeshData,
};

use crate::{
    dist,
    processor::{
        Process,
        Processor,
    },
    source::Mesh,
    Error,
};

impl Process for Mesh {
    fn process(
        &self,
        id: AssetId,
        processor: &mut Processor,
        manifest_path: &Path,
    ) -> Result<(), Error> {
        let path = manifest_path.join(&self.mesh);

        // check if mesh parses correctly
        let reader = BufReader::new(File::open(&path)?);
        let _mesh: MeshData = rmp_serde::from_read(reader)?;

        let filename = format!("{id}.mesh");
        std::fs::copy(&path, processor.dist_path.join(&filename))?;

        processor.dist_manifest.meshes.push(dist::Mesh {
            id,
            mesh: filename,
            label: self.label.clone(),
        });

        Ok(())
    }
}

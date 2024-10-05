use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
};

use kardashev_protocol::assets::{
    AssetId,
    MeshData,
};

use crate::assets::{
    dist,
    processor::ProcessContext,
    source::{
        Manifest,
        Mesh,
    },
    Asset,
    Error,
};

impl Asset for Mesh {
    fn register_dist_type(dist_asset_types: &mut dist::AssetTypes) {
        dist_asset_types.register::<dist::Mesh>();
    }

    fn get_assets(manifest: &Manifest) -> &HashMap<AssetId, Self> {
        &manifest.meshes
    }

    async fn process<'a, 'b: 'a>(
        &'a self,
        id: AssetId,
        context: &'a mut ProcessContext<'b>,
    ) -> Result<(), Error> {
        if !context.processing(id) {
            return Ok(());
        }

        let path = context.input_path(&self.mesh);

        if context.source_path(id, &path)?.is_fresh() {
            tracing::debug!(%id, "not modified since last build. skipping.");
            return Ok(());
        }

        // check if mesh parses correctly
        let reader = BufReader::new(File::open(&path)?);
        let _mesh: MeshData = rmp_serde::from_read(reader)?;

        let filename = format!("{id}.mesh");
        std::fs::copy(&path, context.dist_path.join(&filename))?;

        context.dist_assets.insert(dist::Mesh {
            id,
            mesh: filename,
            label: self.label.clone(),
        });

        context.set_build_time(id);

        Ok(())
    }
}

use std::{
    collections::HashMap,
    fs::File,
    io::BufWriter,
};

use image::{
    ImageFormat,
    ImageReader,
};
use kardashev_protocol::assets::AssetId;

use crate::{
    dist,
    processor::ProcessContext,
    source::{
        Manifest,
        ScaleTo,
        Texture,
    },
    Asset,
    Error,
};

impl Asset for Texture {
    fn get_assets(manifest: &Manifest) -> &HashMap<AssetId, Self> {
        &manifest.textures
    }

    fn process<'a, 'b: 'a>(
        &self,
        id: AssetId,
        context: &'a mut ProcessContext<'b>,
    ) -> Result<(), Error> {
        if !context.processing(id) {
            return Ok(());
        }

        let path = context.input_path(&self.path);

        if context.is_fresh_file(id, &path)? {
            tracing::debug!("not modified since last build. skipping.");
            return Ok(());
        }

        let mut image = ImageReader::open(&path)?.decode()?;

        if let Some(scale_to) = &self.scale_to {
            let new_dimensions = match scale_to {
                ScaleTo {
                    width: Some(width),
                    height: Some(height),
                    ..
                } => [*width, *height],
                ScaleTo {
                    width: Some(width),
                    height: None,
                    ..
                } => {
                    [
                        *width,
                        (*width as f32 / image.width() as f32 * image.height() as f32) as u32,
                    ]
                }
                ScaleTo {
                    width: None,
                    height: Some(height),
                    ..
                } => {
                    [
                        (*height as f32 / image.height() as f32 * image.width() as f32) as u32,
                        *height,
                    ]
                }
                _ => panic!("Either width, height, or both must be specified for scaling"),
            };
            image = image.resize_exact(
                new_dimensions[0],
                new_dimensions[1],
                scale_to.filter.unwrap_or_default().into(),
            );
        }

        if let Some(atlas_builder_id) = self.atlas.clone().unwrap_or_default().into() {
            let atlas_builder = context.atlas_builders.entry(atlas_builder_id).or_default();
            atlas_builder.insert(
                image.to_rgba8(),
                UnfinishedTexture {
                    id,
                    label: self.label.clone(),
                },
            )?;
        }
        else {
            let filename = format!("{id}.png");
            let path = context.dist_path.join(&filename);
            let mut writer = BufWriter::new(File::create(&path)?);
            image.write_to(&mut writer, ImageFormat::Png)?;

            context.dist_manifest.textures.push(dist::Texture {
                id,
                image: filename.clone(),
                label: self.label.clone(),
                size: dist::TextureSize {
                    w: image.width(),
                    h: image.height(),
                },
                crop: None,
                u_edge_mode: None,
                v_edge_mode: None,
            });
        }

        context.set_build_time(id);

        Ok(())
    }
}

#[derive(Debug)]
pub struct UnfinishedTexture {
    pub id: AssetId,
    pub label: Option<String>,
}

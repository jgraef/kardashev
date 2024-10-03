use std::{
    fs::File,
    io::BufWriter,
    path::Path,
};

use color_eyre::eyre::bail;
use image::{
    ImageFormat,
    ImageReader,
};
use kardashev_protocol::assets::AssetId;

use crate::{
    dist,
    processor::{
        input_path,
        Process,
        Processor,
    },
    source::{
        ScaleTo,
        Texture,
    },
    Error,
};

impl Process for Texture {
    fn process(
        &self,
        id: AssetId,
        processor: &mut Processor,
        manifest_path: &Path,
    ) -> Result<(), Error> {
        let path = input_path(manifest_path, &self.path);

        tracing::debug!(%id, label = ?self.label, path = %path.display(), "processing image");

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
                _ => bail!("Either width, height, or both must be specified for scaling"),
            };
            image = image.resize_exact(
                new_dimensions[0],
                new_dimensions[1],
                scale_to.filter.unwrap_or_default().into(),
            );
        }

        if let Some(atlas_builder_id) = self.atlas.clone().unwrap_or_default().into() {
            let atlas_builder = processor
                .atlas_builders
                .entry(atlas_builder_id)
                .or_default();
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
            let path = processor.dist_path.join(&filename);
            let mut writer = BufWriter::new(File::create(&path)?);
            image.write_to(&mut writer, ImageFormat::Png)?;

            processor.dist_manifest.textures.push(dist::Texture {
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

        Ok(())
    }
}

#[derive(Debug)]
pub struct UnfinishedTexture {
    pub id: AssetId,
    pub label: Option<String>,
}

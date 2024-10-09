use std::{
    collections::HashMap,
    fs::File,
    io::BufWriter,
};

use image::ImageReader;
use kardashev_protocol::assets::AssetId;

use crate::assets::{
    dist,
    processor::ProcessContext,
    source::{
        Manifest,
        ScaleTo,
        Texture,
        TextureFileFormat,
    },
    Asset,
    Error,
};

impl Asset for Texture {
    fn register_dist_type(dist_asset_types: &mut dist::AssetTypes) {
        dist_asset_types.register::<dist::Texture>();
    }

    fn get_assets(manifest: &Manifest) -> &HashMap<AssetId, Self> {
        &manifest.textures
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

        let mut image = {
            let path = path.clone();
            tokio::task::spawn_blocking(move || ImageReader::open(path)?.decode())
                .await
                .unwrap()?
        };

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
                _ => {
                    // todo: return an error
                    panic!("Either width, height, or both must be specified for scaling")
                }
            };
            let filter = scale_to.filter.unwrap_or_default().into();
            image = tokio::task::spawn_blocking(move || {
                image.resize_exact(new_dimensions[0], new_dimensions[1], filter)
            })
            .await
            .unwrap();
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
            let size = dist::TextureSize {
                w: image.width(),
                h: image.height(),
            };

            let output_format = self.output_format.unwrap_or_default();
            let filename = format!("{id}.{}", output_format.file_extension());
            let path = context.dist_path.join(&filename);

            match output_format {
                TextureFileFormat::Jpeg
                | TextureFileFormat::Png
                | TextureFileFormat::Gif
                | TextureFileFormat::Webp
                | TextureFileFormat::Tiff => {
                    tokio::task::spawn_blocking(move || {
                        let mut writer = BufWriter::new(File::create(&path)?);
                        image.write_to(&mut writer, output_format.image_format().unwrap())
                    })
                    .await
                    .unwrap()?;
                }
                TextureFileFormat::Ktx2 => {
                    todo!();
                }
            }

            context.dist_assets.insert(dist::Texture {
                id,
                label: self.label.clone(),
                build_time: context.build_time,
                image: filename.clone(),
                size,
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

use std::{
    collections::HashMap,
    fs::File,
    io::BufWriter,
    path::{
        Path,
        PathBuf,
    },
};

use color_eyre::eyre::{
    bail,
    OptionExt,
};
use image::{
    ImageFormat,
    ImageReader,
};
use kardashev_protocol::assets::{
    self as dist,
    AssetId,
};
use walkdir::WalkDir;

use crate::{
    atlas::{
        AtlasBuilder,
        AtlasBuilderId,
    },
    source::{
        Asset,
        Manifest,
        Material,
        ScaleTo,
        Texture,
    },
    Error,
};

#[derive(Debug)]
pub struct Processor {
    dist_path: PathBuf,
    dist_manifest: dist::Manifest,
    atlas_builders: HashMap<AtlasBuilderId, AtlasBuilder<UnfinishedTexture>>,
}

impl Processor {
    pub fn new(dist: impl AsRef<Path>) -> Self {
        Self {
            dist_path: dist.as_ref().to_owned(),
            dist_manifest: dist::Manifest::default(),
            atlas_builders: HashMap::new(),
        }
    }

    pub fn process_directory(&mut self, path: impl AsRef<Path>) -> Result<(), Error> {
        for result in WalkDir::new(path) {
            let entry = result?;

            if entry.file_name() == "Asset.toml" {
                self.process_manifest(entry.path())?;
            }
        }

        Ok(())
    }

    pub fn process_manifest(&mut self, path: impl AsRef<Path>) -> Result<(), Error> {
        let path = path.as_ref();
        let toml = std::fs::read_to_string(path)?;
        let manifest: Manifest = toml::from_str(&toml)?;

        macro_rules! process {
            ($($field:ident),*) => {
                $(
                    for asset in &manifest.$field {
                        asset.process(self, path)?;
                    }
                )*
            };
        }

        process!(texture);
        //process!(texture, material, mesh, model, sound);

        Ok(())
    }

    pub fn finalize(mut self) -> Result<(), Error> {
        for (atlas_builder_id, atlas_builder) in self.atlas_builders {
            let atlas = atlas_builder.finish()?;
            let filename = format!("atlas_{atlas_builder_id}.png");
            let path = self.dist_path.join(&filename);
            let mut writer = BufWriter::new(File::create(&path)?);
            atlas.image.write_to(&mut writer, ImageFormat::Png)?;

            for (data, crop) in atlas.allocations {
                self.dist_manifest.textures.push(dist::Texture {
                    id: data.id,
                    image: filename.clone(),
                    label: data.label,
                    size: dist::TextureSize {
                        w: atlas.image_size[0],
                        h: atlas.image_size[1],
                    },
                    crop: Some(crop),
                    u_edge_mode: None,
                    v_edge_mode: None,
                });
            }
        }

        let writer = BufWriter::new(File::create(self.dist_path.join("assets.json"))?);
        serde_json::to_writer_pretty(writer, &self.dist_manifest)?;

        Ok(())
    }
}

pub trait Process {
    fn process(&self, processor: &mut Processor, manifest_path: &Path) -> Result<(), Error>;
}

impl Process for Asset<Texture> {
    fn process(&self, processor: &mut Processor, manifest_path: &Path) -> Result<(), Error> {
        let path = or_default_filename(
            self.inner.path.as_deref(),
            self.id,
            self.label.as_deref(),
            manifest_path,
            &["png", "jpg", "jpeg", "tif", "webp"],
        )?;

        tracing::debug!(id = %self.id, label = ?self.label, path = %path.display(), "processing image");
        let mut image = ImageReader::open(&path)?.decode()?;

        if let Some(scale_to) = &self.inner.scale_to {
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

        if let Some(atlas_builder_id) = self.inner.atlas.clone().unwrap_or_default().into() {
            let atlas_builder = processor
                .atlas_builders
                .entry(atlas_builder_id)
                .or_default();
            atlas_builder.insert(
                image.to_rgba8(),
                UnfinishedTexture {
                    id: self.id,
                    label: self.label.clone(),
                },
            )?;
        }
        else {
            // todo: copy file to dist folder and write metadata to dist_manifest
            todo!();
        }

        Ok(())
    }
}

#[derive(Debug)]
struct UnfinishedTexture {
    id: AssetId,
    label: Option<String>,
}

impl Process for Asset<Material> {
    fn process(&self, processor: &mut Processor, manifest_path: &Path) -> Result<(), Error> {
        todo!()
    }
}

fn or_default_filename(
    path: Option<impl AsRef<Path>>,
    id: AssetId,
    label: Option<&str>,
    manifest_path: &Path,
    extensions: &[&str],
) -> Result<PathBuf, Error> {
    let dir = manifest_path
        .parent()
        .ok_or_eyre("manifest path has no parent")?;

    if let Some(path) = path {
        return Ok(dir.join(path));
    }

    let derive_path = |prefix: &str, ext: &str| {
        let path = dir.join(format!("{prefix}.{ext}"));
        path.exists().then_some(path)
    };

    let id_str = id.to_string();

    for &ext in extensions {
        if let Some(id_path) = derive_path(&id_str, ext) {
            return Ok(id_path);
        }
        if let Some(label) = label {
            if let Some(label_path) = derive_path(label, ext) {
                return Ok(label_path);
            }
        }
    }

    bail!("Filename for asset {id} could not be determined.");
}

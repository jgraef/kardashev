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
        Manifest,
        Material,
        ScaleTo,
        Shader,
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
    pub fn new(dist_path: impl AsRef<Path>) -> Result<Self, Error> {
        let dist_path = dist_path.as_ref();
        std::fs::create_dir_all(dist_path)?;
        Ok(Self {
            dist_path: dist_path.to_owned(),
            dist_manifest: dist::Manifest::default(),
            atlas_builders: HashMap::new(),
        })
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
                    for (id, asset) in &manifest.$field {
                        asset.process(*id, self, path)?;
                    }
                )*
            };
        }

        process!(textures, shaders);
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
    fn process(
        &self,
        id: AssetId,
        processor: &mut Processor,
        manifest_path: &Path,
    ) -> Result<(), Error>;
}

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
struct UnfinishedTexture {
    id: AssetId,
    label: Option<String>,
}

impl Process for Material {
    fn process(
        &self,
        _id: AssetId,
        _processor: &mut Processor,
        _manifest_path: &Path,
    ) -> Result<(), Error> {
        todo!()
    }
}

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

fn input_path(manifest_path: impl AsRef<Path>, file_path: impl AsRef<Path>) -> PathBuf {
    manifest_path
        .as_ref()
        .parent()
        .expect("manifest path has no parent directory")
        .join(file_path)
}

// unused
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

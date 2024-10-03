pub mod material;
pub mod mesh;
pub mod shader;
pub mod texture;

use std::{
    collections::HashMap,
    fs::File,
    io::{
        BufReader,
        BufWriter,
    },
    path::{
        Path,
        PathBuf,
    },
};

use color_eyre::eyre::{
    bail,
    OptionExt,
};
use image::ImageFormat;
use kardashev_protocol::assets::{
    self as dist,
    AssetId,
};
use walkdir::WalkDir;

use self::texture::UnfinishedTexture;
use crate::{
    atlas::{
        AtlasBuilder,
        AtlasBuilderId,
    },
    build_info::BuildInfo,
    source::Manifest,
    Error,
};

#[derive(Debug)]
pub struct Processor {
    dist_path: PathBuf,
    dist_manifest: dist::Manifest,
    atlas_builders: HashMap<AtlasBuilderId, AtlasBuilder<UnfinishedTexture>>,
    build_info: BuildInfo,
}

impl Processor {
    pub fn new(dist_path: impl AsRef<Path>) -> Result<Self, Error> {
        let dist_path = dist_path.as_ref();
        std::fs::create_dir_all(dist_path)?;

        let build_info_path = dist_path.join("build_info.json");
        let build_info = build_info_path
            .exists()
            .then(|| {
                let reader = BufReader::new(File::open(&build_info_path)?);
                Ok::<_, Error>(serde_json::from_reader(reader)?)
            })
            .transpose()?
            .unwrap_or_default();

        Ok(Self {
            dist_path: dist_path.to_owned(),
            dist_manifest: dist::Manifest::default(),
            atlas_builders: HashMap::new(),
            build_info,
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

        process!(textures, materials, meshes, shaders);

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

        let writer = BufWriter::new(File::create(self.dist_path.join("build_info.json"))?);
        serde_json::to_writer_pretty(writer, &self.build_info)?;

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

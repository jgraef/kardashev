use std::{
    any::type_name,
    collections::{
        HashMap,
        HashSet,
    },
    fmt::Debug,
    fs::File,
    io::{
        BufReader,
        BufWriter,
    },
    marker::PhantomData,
    ops::Deref,
    path::{
        Path,
        PathBuf,
    },
};

use chrono::{
    DateTime,
    Utc,
};
use image::ImageFormat;
use walkdir::WalkDir;

use crate::{
    atlas::{
        AtlasBuilder,
        AtlasBuilderId,
    },
    build_info::BuildInfo,
    dist,
    source::Manifest,
    texture::UnfinishedTexture,
    Asset,
    AssetId,
    Error,
};

#[derive(Debug)]
pub struct Processor {
    asset_types: Vec<DynAssetType>,
    source: Source,
    dist_path: PathBuf,
    dist_manifest: dist::Manifest,
    build_info: BuildInfo,
}

impl Processor {
    pub fn new(dist_path: impl AsRef<Path>) -> Result<Self, Error> {
        use crate::source;

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
            asset_types: vec![
                DynAssetType::new::<source::Material>(),
                DynAssetType::new::<source::Texture>(),
                DynAssetType::new::<source::Mesh>(),
                DynAssetType::new::<source::Shader>(),
            ],
            source: Source::default(),
            dist_path: dist_path.to_owned(),
            dist_manifest: dist::Manifest::default(),
            build_info,
        })
    }

    pub fn register_asset_type<A: Asset>(&mut self) {
        self.asset_types.push(DynAssetType::new::<A>());
    }

    pub fn add_directory(&mut self, path: impl AsRef<Path>) -> Result<(), Error> {
        for result in WalkDir::new(path) {
            let entry = result?;

            if entry.file_name() == "Asset.toml" {
                self.add_manifest(entry.path())?;
            }
        }

        Ok(())
    }

    pub fn add_manifest(&mut self, path: impl AsRef<Path>) -> Result<(), Error> {
        let path = path.as_ref();

        tracing::info!(path = %path.display(), "adding manifest file");

        let toml = std::fs::read_to_string(path)?;
        let manifest: Manifest = toml::from_str(&toml)?;

        let index = self.source.manifests.len();
        let mut num_assets = 0;

        for asset_type in &self.asset_types {
            for asset_id in asset_type.get_asset_ids(&manifest) {
                self.source.refs.insert(asset_id, (*asset_type, index));
                num_assets += 1;
            }
        }

        if num_assets == 0 {
            tracing::warn!(path = %path.display(), "manifest with no recognized assets");
        }

        self.source.manifests.push((path.to_owned(), manifest));

        Ok(())
    }

    pub fn process(&mut self) -> Result<Processed, Error> {
        let build_time = Utc::now();
        let mut processed = HashSet::new();
        let mut changed = HashSet::new();
        let mut atlas_builders = HashMap::new();

        for (path, manifest) in &self.source.manifests {
            tracing::info!(path = %path.display(), "processing manifest file");

            for asset_type in &self.asset_types {
                for id in asset_type.get_asset_ids(manifest) {
                    tracing::info!(%id, asset_type = asset_type.type_name(), "processing asset");

                    let mut context = ProcessContext {
                        manifest_path: &path,
                        source: &self.source,
                        dist_path: &self.dist_path,
                        dist_manifest: &mut self.dist_manifest,
                        build_info: &mut self.build_info,
                        atlas_builders: &mut atlas_builders,
                        build_time,
                        processed: &mut processed,
                        changed: &mut changed,
                    };
                    asset_type.process(&mut context, id)?;
                }
            }
        }

        for (atlas_builder_id, atlas_builder) in atlas_builders {
            tracing::info!(%atlas_builder_id, "building texture atlas");

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

        let path = self.dist_path.join("assets.json");
        tracing::info!(path = %path.display(), "writing dist manifest");
        let writer = BufWriter::new(File::create(path)?);
        serde_json::to_writer_pretty(writer, &self.dist_manifest)?;

        let path = self.dist_path.join("build_info.json");
        tracing::info!(path = %path.display(), "writing build info");
        let writer = BufWriter::new(File::create(path)?);
        serde_json::to_writer_pretty(writer, &self.build_info)?;

        Ok(Processed { changed })
    }
}

#[derive(Clone, Debug)]
pub struct Processed {
    pub changed: HashSet<AssetId>,
}

#[derive(Clone, Debug, Default)]
pub struct Source {
    manifests: Vec<(PathBuf, Manifest)>,
    refs: HashMap<AssetId, (DynAssetType, usize)>,
}

impl Source {
    pub fn get_asset<A: Asset>(&self, id: AssetId) -> Option<&A> {
        let index = self.refs.get(&id)?.1;
        let (_, manifest) = self.manifests.get(index)?;
        A::get_assets(manifest).get(&id)
    }
}

#[derive(Debug)]
pub struct ProcessContext<'a> {
    pub manifest_path: &'a Path,
    pub source: &'a Source,
    pub dist_path: &'a Path,
    pub dist_manifest: &'a mut dist::Manifest,
    pub build_info: &'a mut BuildInfo,
    pub atlas_builders: &'a mut HashMap<AtlasBuilderId, AtlasBuilder<UnfinishedTexture>>,
    pub build_time: DateTime<Utc>,
    pub processed: &'a mut HashSet<AssetId>,
    pub changed: &'a mut HashSet<AssetId>,
}

impl<'a> ProcessContext<'a> {
    pub fn input_path(&self, file_path: impl AsRef<Path>) -> PathBuf {
        self.manifest_path
            .parent()
            .expect("manifest path has no parent directory")
            .join(file_path)
    }

    pub fn set_build_time(&mut self, id: AssetId) {
        self.build_info.build_times.insert(id, self.build_time);
        self.changed.insert(id);
    }

    pub fn is_fresh_file(&self, id: AssetId, path: impl AsRef<Path>) -> Result<bool, Error> {
        let modified_time = file_modified_timestamp(path)?;
        Ok(self.is_fresh(id, modified_time))
    }

    pub fn is_fresh_dependency(&self, id: AssetId, dependency: AssetId) -> bool {
        let dependency_build_time = self.build_info.build_times.get(&dependency);
        dependency_build_time.map_or(false, |dependency_build_time| {
            self.is_fresh(id, *dependency_build_time)
        })
    }

    pub fn is_fresh(&self, id: AssetId, time: DateTime<Utc>) -> bool {
        let build_time = self.build_info.build_times.get(&id);
        build_time.map_or(false, |build_time| build_time > &time)
    }

    pub fn processing(&mut self, id: AssetId) -> bool {
        if self.processed.contains(&id) {
            tracing::debug!(%id, "asset already processed. skipping.");
            false
        }
        else {
            self.processed.insert(id);
            true
        }
    }
}

pub fn file_modified_timestamp(path: impl AsRef<Path>) -> Result<DateTime<Utc>, Error> {
    let path = path.as_ref();
    let metadata = path.metadata()?;
    Ok(metadata.modified()?.into())
}

#[derive(Clone, Copy)]
struct DynAssetType {
    asset_type: &'static dyn DynAssetTypeTrait,
}

impl DynAssetType {
    pub fn new<A: Asset>() -> Self {
        Self {
            asset_type: &DynAssetTypeImpl {
                _ty: PhantomData::<A>,
            },
        }
    }
}

impl Debug for DynAssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynAssetType")
            .field("A", &self.asset_type.type_name())
            .finish()
    }
}

impl Deref for DynAssetType {
    type Target = dyn DynAssetTypeTrait;

    fn deref(&self) -> &Self::Target {
        self.asset_type
    }
}

trait DynAssetTypeTrait {
    fn type_name(&self) -> &'static str;
    fn get_asset_ids(&self, manifest: &Manifest) -> Vec<AssetId>;
    fn process<'a, 'b: 'a>(
        &self,
        context: &'a mut ProcessContext<'b>,
        asset_id: AssetId,
    ) -> Result<(), Error>;
}

struct DynAssetTypeImpl<A: Asset> {
    _ty: PhantomData<A>,
}

impl<A: Asset> DynAssetTypeTrait for DynAssetTypeImpl<A> {
    fn type_name(&self) -> &'static str {
        type_name::<A>()
    }

    fn get_asset_ids(&self, manifest: &Manifest) -> Vec<AssetId> {
        A::get_assets(manifest).keys().copied().collect()
    }

    fn process<'a, 'b: 'a>(
        &self,
        context: &'a mut ProcessContext<'b>,
        id: AssetId,
    ) -> Result<(), Error> {
        let asset = context.source.get_asset::<A>(id).unwrap();
        asset.process(id, context)
    }
}

use std::{
    any::type_name,
    collections::{
        HashMap,
        HashSet,
    },
    fmt::Debug,
    fs::File,
    future::Future,
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
    pin::Pin, time::Duration,
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
    build_info::{
        BuildInfo,
        CompressionFormat,
    },
    dist,
    source::Manifest,
    texture::UnfinishedTexture,
    watch::{ChangedPaths, WatchSources},
    Asset,
    AssetId,
    Error,
};

#[derive(Debug)]
pub struct Processor {
    asset_types: Vec<DynAssetType>,
    source: Source,
    dist_path: PathBuf,
    build_info: BuildInfo,
    precompress: HashSet<CompressionFormat>,
    watch_sources: Option<WatchSources>,
}

impl Processor {
    pub fn new(dist_path: impl AsRef<Path>) -> Result<Self, Error> {
        use crate::source;

        let dist_path = dist_path.as_ref();
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
                DynAssetType::new::<source::Wasm>(),
            ],
            source: Source::default(),
            dist_path: dist_path.to_owned(),
            build_info,
            precompress: HashSet::new(),
            watch_sources: None,
        })
    }

    pub fn watch_source_files(&mut self) -> Result<(), Error> {
        if self.watch_sources.is_none() {
            self.watch_sources = Some(WatchSources::new()?);
        }
        Ok(())
    }

    pub async fn wait_for_changes(&mut self, debounce: Option<Duration>) -> Option<ChangedPaths> {
        self.watch_sources.as_mut()?.next_changes(debounce).await
    }

    pub fn register_asset_type<A: Asset>(&mut self) {
        self.asset_types.push(DynAssetType::new::<A>());
    }

    pub fn precompress(&mut self, format: CompressionFormat) {
        self.precompress.insert(format);
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

        if let Some(watch_sources) = &mut self.watch_sources {
            watch_sources.add_manifest_path(path.canonicalize()?)?;
        }

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

    pub async fn process(&mut self, clean: bool) -> Result<Processed, Error> {
        let build_time = Utc::now();
        let mut processed = HashSet::new();
        let mut changed = HashSet::new();
        let mut atlas_builders = HashMap::new();
        let mut watch_sources = self.watch_sources.as_ref().map(|_| HashSet::new());

        // create dist path, if it doesn't exist already
        std::fs::create_dir_all(&self.dist_path)?;

        // if this is a clean build, we need to clear build times
        if clean {
            self.build_info.build_times.clear();
        }

        // load dist manifest if it exists and this isn't a clean build
        let path = self.dist_path.join("assets.json");
        let mut dist_assets = (path.exists() && !clean)
            .then(|| {
                let reader = BufReader::new(File::open(&path)?);
                let dist_manifest: dist::Manifest = serde_json::from_reader(reader)?;
                let mut dist_asset_types = dist::AssetTypes::default();
                dist_asset_types.with_builtin();
                for asset_type in &self.asset_types {
                    asset_type.register_dist_type(&mut dist_asset_types);
                }
                Ok::<_, Error>(dist_manifest.assets.parse(&dist_asset_types)?)
            })
            .transpose()?
            .unwrap_or_default();

        for (path, manifest) in &self.source.manifests {
            tracing::info!(path = %path.display(), "processing manifest file");

            for asset_type in &self.asset_types {
                for id in asset_type.get_asset_ids(manifest) {
                    tracing::info!(%id, asset_type = asset_type.type_name(), "processing asset");

                    let mut context = ProcessContext {
                        manifest_path: &path,
                        source: &self.source,
                        dist_path: &self.dist_path,
                        dist_assets: &mut dist_assets,
                        build_info: &mut self.build_info,
                        atlas_builders: &mut atlas_builders,
                        build_time,
                        processed: &mut processed,
                        changed: &mut changed,
                        precompress: &self.precompress,
                        watch_sources: watch_sources.as_mut(),
                    };
                    asset_type.process(&mut context, id).await?;
                }
            }
        }

        // update file watcher
        match (&mut self.watch_sources, watch_sources) {
            (Some(watch_sources), Some(new)) => {
                watch_sources.set_source_paths(new)?;
            }
            _ => {}
        }

        // remove assets that were not generated this time
        let dist_asset_ids = dist_assets.all_asset_ids().collect::<Vec<_>>();
        for asset_id in dist_asset_ids {
            if !processed.contains(&asset_id) {
                tracing::info!(%asset_id, "removing old asset");
                dist_assets.remove(asset_id);
            }
        }

        // collect files from dist manifest
        let mut files = dist_assets
            .all_files()
            .into_iter()
            .map(PathBuf::from)
            .collect::<HashSet<_>>();

        // add texture atlasses
        // todo: texture atlasses should become an (source) asset that just creates a
        // texture asset
        for (atlas_builder_id, atlas_builder) in atlas_builders {
            tracing::info!(%atlas_builder_id, "building texture atlas");

            let atlas = atlas_builder.finish()?;
            let filename = format!("atlas_{atlas_builder_id}.png");
            files.insert(PathBuf::from(&filename));
            let path = self.dist_path.join(&filename);
            let mut writer = BufWriter::new(File::create(&path)?);
            atlas.image.write_to(&mut writer, ImageFormat::Png)?;

            for (data, crop) in atlas.allocations {
                dist_assets.insert(dist::Texture {
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

        // write dist manifest
        let dist_manifest = dist::Manifest {
            build_time,
            assets: dist_assets.blob(),
        };
        files.insert(PathBuf::from("assets.json"));
        let path = self.dist_path.join("assets.json");
        tracing::info!(path = %path.display(), "writing dist manifest");
        let writer = BufWriter::new(File::create(&path)?);
        serde_json::to_writer_pretty(writer, &dist_manifest)?;

        // write build info
        files.insert(PathBuf::from("build_info.json"));
        let path = self.dist_path.join("build_info.json");
        tracing::info!(path = %path.display(), "writing build info");
        let writer = BufWriter::new(File::create(&path)?);
        serde_json::to_writer_pretty(writer, &self.build_info)?;

        // cleanup files
        for result in std::fs::read_dir(&self.dist_path)? {
            let entry = result?;
            let filename = PathBuf::from(entry.file_name());
            if !files.contains(&filename) {
                tracing::info!(file = %filename.display(), "cleaning up file");
                std::fs::remove_file(self.dist_path.join(&filename))?;
            }
        }

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
    pub dist_assets: &'a mut dist::Assets,
    pub build_info: &'a mut BuildInfo,
    pub atlas_builders: &'a mut HashMap<AtlasBuilderId, AtlasBuilder<UnfinishedTexture>>,
    pub build_time: DateTime<Utc>,
    pub processed: &'a mut HashSet<AssetId>,
    pub changed: &'a mut HashSet<AssetId>,
    pub precompress: &'a HashSet<CompressionFormat>,
    pub watch_sources: Option<&'a mut HashSet<PathBuf>>,
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

    fn freshness(&self, id: AssetId, time: DateTime<Utc>) -> Freshness {
        let build_time = self.build_info.build_times.get(&id);
        build_time
            .and_then(|build_time| (build_time > &time).then_some(Freshness::Fresh))
            .unwrap_or(Freshness::Stale)
    }

    pub fn source_path(&mut self, id: AssetId, path: impl AsRef<Path>) -> Result<Freshness, Error> {
        let path = path.as_ref();
        
        if let Some(watch_sources) = &mut self.watch_sources {
            watch_sources.insert(path.canonicalize()?.to_owned());
        }

        let modified_time = path_modified_timestamp(path)?;
        Ok(self.freshness(id, modified_time))
    }

    pub fn source_asset(&self, id: AssetId, dependency: AssetId) -> Freshness {
        let dependency_build_time = self.build_info.build_times.get(&dependency);
        dependency_build_time.map_or(Freshness::Stale, |dependency_build_time| {
            self.freshness(id, *dependency_build_time)
        })
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

    pub fn precompress(&mut self, filename: &str) -> Result<(), Error> {
        for &format in self.precompress {
            let _compressed = compress(format, &self.dist_path, filename)?;
            // todo
            //self.build_info
            //    .precompressed
            //    .insert(filename.to_owned(), Precompressed { format,
            // compressed });
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Freshness {
    Fresh,
    Stale,
}

impl Freshness {
    pub fn and(&mut self, other: Freshness) {
        match other {
            Freshness::Fresh => {}
            Freshness::Stale => *self = Freshness::Stale,
        }
    }

    pub fn is_fresh(&self) -> bool {
        *self == Freshness::Fresh
    }

    pub fn is_stale(&self) -> bool {
        *self == Freshness::Stale
    }
}

fn compress(
    format: CompressionFormat,
    dist_path: impl AsRef<Path>,
    filename: &str,
) -> Result<String, Error> {
    let dist_path = dist_path.as_ref();
    tracing::debug!(filename, ?format, "compressing");

    Ok(match format {
        CompressionFormat::Gzip => {
            let compressed = format!("{filename}.gz");
            let mut writer = libflate::gzip::Encoder::new(BufWriter::new(File::create(
                dist_path.join(&compressed),
            )?))?;
            let mut reader = BufReader::new(File::open(dist_path.join(&filename))?);
            std::io::copy(&mut reader, &mut writer)?;
            compressed
        }
    })
}

pub fn path_modified_timestamp(path: impl AsRef<Path>) -> Result<DateTime<Utc>, Error> {
    let path = path.as_ref();

    let metadata = path.metadata()?;
    let mut modified_time: DateTime<Utc> = metadata.modified()?.into();

    if metadata.is_dir() {
        for result in WalkDir::new(path) {
            let entry = result?;
            let metadata = entry.metadata()?;
            modified_time = modified_time.max(metadata.modified()?.into())
        }
    }

    Ok(modified_time)
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

trait DynAssetTypeTrait: Send + Sync + 'static {
    fn type_name(&self) -> &'static str;
    fn get_asset_ids(&self, manifest: &Manifest) -> Vec<AssetId>;
    fn process<'a, 'b: 'a>(
        &'a self,
        context: &'a mut ProcessContext<'b>,
        asset_id: AssetId,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + Sync + 'a>>;
    fn register_dist_type(&self, dist_asset_types: &mut dist::AssetTypes);
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
        &'a self,
        context: &'a mut ProcessContext<'b>,
        asset_id: AssetId,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + Sync + 'a>> {
        let asset = context.source.get_asset::<A>(asset_id).unwrap();
        Box::pin(asset.process(asset_id, context))
    }

    fn register_dist_type(&self, dist_asset_types: &mut dist::AssetTypes) {
        A::register_dist_type(dist_asset_types);
    }
}

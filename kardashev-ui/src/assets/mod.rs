mod image_load;

use std::collections::HashMap;

use kardashev_client::AssetClient;
use kardashev_protocol::assets::{
    self as dist,
    AssetId,
};
use tokio::sync::{
    mpsc,
    oneshot,
};

pub use self::image_load::{
    load_image,
    LoadImage,
    LoadImageError,
};
use crate::utils::spawn_local_and_handle_error;

#[derive(Debug, thiserror::Error)]
#[error("asset loader error")]
pub enum Error {
    Reqwest(#[from] reqwest::Error),
    AssetNotFound {
        asset_id: AssetId,
    },
    UnexpectedAssetType {
        asset_id: AssetId,
        unexpected_type: AssetType,
        expected_type: AssetType,
    },
    ImageLoad(#[from] image_load::LoadImageError),
    Graphics(#[from] crate::graphics::Error),
    Client(#[from] kardashev_client::Error),
}

#[derive(Debug)]
pub struct Assets {
    tx_command: mpsc::Sender<Command>,
}

impl Assets {
    pub fn new(client: AssetClient) -> Self {
        let (tx_command, rx_command) = mpsc::channel(16);
        Reactor::spawn(client, rx_command);
        Self { tx_command }
    }
}

struct Manifest {
    dist: dist::Manifest,
    refs: HashMap<AssetId, (AssetType, usize)>,
}

impl Manifest {
    pub fn get<A: ManifestAsset>(&self, asset_id: AssetId) -> Result<&A, Error> {
        let (asset_type, index) = self
            .refs
            .get(&asset_id)
            .copied()
            .ok_or_else(|| Error::AssetNotFound { asset_id })?;

        if asset_type != A::ASSET_TYPE {
            return Err(Error::UnexpectedAssetType {
                asset_id,
                unexpected_type: asset_type,
                expected_type: A::ASSET_TYPE,
            });
        }

        Ok(A::get(&self.dist, index))
    }
}

impl From<dist::Manifest> for Manifest {
    fn from(dist: dist::Manifest) -> Self {
        let mut refs = HashMap::new();

        for (i, texture) in dist.textures.iter().enumerate() {
            refs.insert(texture.id, (AssetType::Texture, i));
        }

        Self { dist, refs }
    }
}

trait ManifestAsset: 'static {
    const ASSET_TYPE: AssetType;
    fn get<'a>(dist: &'a dist::Manifest, index: usize) -> &'a Self;
}

macro_rules! impl_asset_types {
    {$($field:ident: $dist_ty:ty, $ty_const:expr;)*} => {
        fn make_refs(dist: &dist::Manifest) -> HashMap<AssetId, (AssetType, usize)> {
            let mut refs = HashMap::new();

            $(
                for (i, asset) in dist.$field.iter().enumerate() {
                    refs.insert(
                        asset.id,
                        (
                            $ty_const,
                            i,
                        ),
                    );
                }
            )*

            refs
        }

        $(
            impl ManifestAsset for $dist_ty {
                const ASSET_TYPE: AssetType = $ty_const;
                fn get<'a>(dist: &'a dist::Manifest, index: usize) -> &'a Self {
                    &dist.$field[index]
                }
            }
        )*
    };
}

impl_asset_types! {
    textures: dist::Texture, AssetType::Texture;
    materials: dist::Material, AssetType::Material;
}

struct Reactor {
    client: AssetClient,
    manifest: Manifest,
    rx_command: mpsc::Receiver<Command>,
}

impl Reactor {
    async fn new(client: AssetClient, rx_command: mpsc::Receiver<Command>) -> Result<Self, Error> {
        let manifest = client.get_manifest().await?;

        Ok(Self {
            client,
            manifest: manifest.into(),
            rx_command,
        })
    }

    fn spawn(client: AssetClient, rx_command: mpsc::Receiver<Command>) {
        spawn_local_and_handle_error(async move {
            let reactor = Self::new(client, rx_command).await?;
            reactor.run().await
        });
    }

    async fn run(mut self) -> Result<(), Error> {
        let mut events = self.client.events().await?;

        loop {
            tokio::select! {
                command_opt = self.rx_command.recv() => {
                    let Some(command) = command_opt else { break; };
                    self.handle_command(command).await?;
                }
                event_result = events.next() => {
                    self.handle_event(event_result?).await?;
                }
            }
        }

        Ok(())
    }

    async fn handle_command(&mut self, command: Command) -> Result<(), Error> {
        match command {
            Command::LoadMaterial {
                asset_id,
                tx_result,
            } => {
                let result = self.load_material(asset_id).await;
                let _ = tx_result.send(result);
            }
        }

        Ok(())
    }

    async fn handle_event(&mut self, event: dist::Message) -> Result<(), Error> {
        match event {
            dist::Message::Reload { asset_id: _ } => {
                // todo: the specified asset was changed and can be reloaded
            }
        }

        Ok(())
    }

    async fn load_material(&self, asset_id: AssetId) -> Result<(), Error> {
        let _material = self.manifest.get::<dist::Material>(asset_id)?;
        todo!();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetType {
    Texture,
    Material,
    Mesh,
    Model,
}

#[derive(Debug)]
enum Command {
    LoadMaterial {
        asset_id: AssetId,
        tx_result: oneshot::Sender<Result<(), Error>>,
    },
}

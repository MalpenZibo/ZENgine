use crossbeam_channel::{Receiver, Sender, TryRecvError};
use downcast_rs::{impl_downcast, Downcast};
use rustc_hash::FxHashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use std::{any::TypeId, path::Path};
use zengine_ecs::system::{Res, ResMut};
use zengine_macro::Resource;

use crate::assets::Assets;
use crate::handle::{HandleEvent, HandleEventChannel, HandleId};
use crate::io_task;
use crate::{
    assets::{Asset, AssetPath},
    handle::Handle,
};

pub struct LoaderContext<'a> {
    asset: Option<Box<dyn Asset>>,
    pub path: &'a Path,
}
impl<'a> LoaderContext<'a> {
    pub fn set_asset<T: Asset>(&mut self, asset: T) {
        self.asset.replace(Box::new(asset));
    }
}

pub trait Loader: Send + Sync + std::fmt::Debug + 'static {
    fn load(&self, data: Vec<u8>, context: &mut LoaderContext);

    fn extension(&self) -> &[&str];
}

pub struct AssetCreateEvent<T> {
    pub id: HandleId,
    pub asset: T,
}

pub enum AssetEvent<T> {
    Create(AssetCreateEvent<T>),
    Destroy(HandleId),
}

#[derive(Debug)]
pub struct AssetChannel<T> {
    pub sender: Sender<AssetEvent<T>>,
    pub receiver: Receiver<AssetEvent<T>>,
}

impl<T> Default for AssetChannel<T> {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self { sender, receiver }
    }
}

trait AnyAssetChannel: Downcast + Sync + Send + std::fmt::Debug + 'static {
    fn create(&self, id: HandleId, asset: Box<dyn Asset>);

    fn destroy(&self, id: HandleId);
}
impl_downcast!(AnyAssetChannel);

impl<T: Asset> AnyAssetChannel for AssetChannel<T> {
    fn create(&self, id: HandleId, asset: Box<dyn Asset>) {
        let asset = asset.downcast::<T>().unwrap_or_else(|_| {
            panic!("Failed to downcast asset to {}", std::any::type_name::<T>())
        });
        self.sender
            .send(AssetEvent::Create(AssetCreateEvent { id, asset: *asset }))
            .unwrap();
    }

    fn destroy(&self, id: HandleId) {
        self.sender.send(AssetEvent::Destroy(id)).unwrap();
    }
}

#[derive(Resource, Debug)]
pub struct AssetManager {
    loaders: Vec<Arc<dyn Loader>>,
    extension_to_loader: FxHashMap<String, usize>,
    asset_channels: Arc<RwLock<FxHashMap<TypeId, Box<dyn AnyAssetChannel>>>>,
    asset_handle_event_channel: HandleEventChannel,
    asset_handle_ref_count: FxHashMap<HandleId, usize>,
}

impl AssetManager {
    pub fn load<T: Asset, P: Into<AssetPath>>(&mut self, file_path: P) -> Handle<T> {
        let asset_path = file_path.into();
        let mut hasher = ahash::AHasher::default();
        asset_path.path.hash(&mut hasher);
        let id: u64 = hasher.finish();
        let type_id = TypeId::of::<T>();

        let handle_id: HandleId = (type_id, id);

        let loader = self
            .find_loader(&asset_path.extension)
            .expect("Asset loader not found");

        let asset_channels = self.asset_channels.clone();

        io_task::spawn(async move {
            cfg_if::cfg_if! {
                if #[cfg(target_arch = "wasm32")] {
                    let data = crate::wasm_asset_io::load(&asset_path.path).await;
                } else {
                    let data = crate::file_asset_io::load(&asset_path.path).await;
                }
            }

            let mut context = LoaderContext {
                asset: None,
                path: asset_path.path.as_path(),
            };

            loader.load(data, &mut context);

            let asset_channels = asset_channels.read().unwrap();
            let asset_channel = asset_channels.get(&type_id).unwrap();

            asset_channel.create(handle_id, context.asset.unwrap());
        });

        Handle::strong(handle_id, self.asset_handle_event_channel.sender.clone())
    }

    pub fn register_asset_type<T: Asset>(&self) -> Assets<T> {
        let type_id = TypeId::of::<T>();
        self.asset_channels
            .write()
            .unwrap()
            .insert(type_id, Box::new(AssetChannel::<T>::default()));

        Assets::new(self.asset_handle_event_channel.sender.clone())
    }

    pub fn register_loader<T: Loader>(&mut self, loader: T) {
        let index = self.loaders.len();
        for e in loader.extension() {
            self.extension_to_loader.insert(e.to_string(), index);
        }
        self.loaders.push(Arc::new(loader));
    }

    fn find_loader(&self, extension: &str) -> Option<Arc<dyn Loader>> {
        self.extension_to_loader
            .get(extension)
            .and_then(|index| self.loaders.get(*index).cloned())
    }
}

pub fn update_asset_storage<T: Asset>(
    asset_manager: Res<AssetManager>,
    mut assets: ResMut<Assets<T>>,
) {
    let type_id = TypeId::of::<T>();
    let asset_channels = asset_manager.asset_channels.read().unwrap();
    let asset_channel = asset_channels.get(&type_id).unwrap();
    let asset_channel = asset_channel.downcast_ref::<AssetChannel<T>>().unwrap();

    loop {
        match asset_channel.receiver.try_recv() {
            Ok(AssetEvent::Create(AssetCreateEvent { id, asset })) => {
                assets.set(&id, asset);
            }
            Ok(AssetEvent::Destroy(id)) => {
                assets.remove(&id);
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => panic!("Asset channel disconnected"),
        }
    }
}

pub fn update_ref_count(mut asset_manager: ResMut<AssetManager>) {
    loop {
        match asset_manager.asset_handle_event_channel.receiver.try_recv() {
            Ok(HandleEvent::Increment(id)) => {
                *asset_manager.asset_handle_ref_count.entry(id).or_insert(0) += 1;
            }
            Ok(HandleEvent::Decrement(id)) => {
                *asset_manager.asset_handle_ref_count.entry(id).or_insert(0) -= 1;
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => panic!("AssetHandle channel disconnected"),
        }
    }
}

pub fn destroy_unused_assets(mut asset_manager: ResMut<AssetManager>) {
    for k in asset_manager
        .asset_handle_ref_count
        .iter()
        .filter_map(|(k, v)| if *v == 0 { Some(*k) } else { None })
        .collect::<Vec<HandleId>>()
    {
        asset_manager.asset_handle_ref_count.remove(&k);
        let asset_channels = asset_manager.asset_channels.read().unwrap();
        let asset_channel = asset_channels.get(&k.0).unwrap();
        asset_channel.destroy(k);
    }
}

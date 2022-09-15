use crossbeam_channel::{Receiver, Sender, TryRecvError};
use downcast_rs::{impl_downcast, Downcast};
use rustc_hash::FxHashMap;
use std::sync::{Arc, RwLock};
use std::{any::TypeId, path::Path};
use zengine_ecs::system::{Res, ResMut};
use zengine_macro::Resource;

use crate::assets::Assets;
use crate::handle::{HandleId, HandleRef, HandleRefChannel};
use crate::io::AssetIo;
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

pub struct AssetCreateCommand<T> {
    pub id: HandleId,
    pub asset: T,
}

pub enum AssetCommand<T> {
    Create(AssetCreateCommand<T>),
    Destroy(HandleId),
}

#[derive(Debug)]
pub struct AssetCommandChannel<T> {
    pub sender: Sender<AssetCommand<T>>,
    pub receiver: Receiver<AssetCommand<T>>,
}

impl<T> Default for AssetCommandChannel<T> {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self { sender, receiver }
    }
}

trait AnyAssetCommandChannel: Downcast + Sync + Send + std::fmt::Debug + 'static {
    fn create(&self, id: HandleId, asset: Box<dyn Asset>);

    fn destroy(&self, id: HandleId);
}
impl_downcast!(AnyAssetCommandChannel);

impl<T: Asset> AnyAssetCommandChannel for AssetCommandChannel<T> {
    fn create(&self, id: HandleId, asset: Box<dyn Asset>) {
        let asset = asset.downcast::<T>().unwrap_or_else(|_| {
            panic!("Failed to downcast asset to {}", std::any::type_name::<T>())
        });
        self.sender
            .send(AssetCommand::Create(AssetCreateCommand {
                id,
                asset: *asset,
            }))
            .unwrap();
    }

    fn destroy(&self, id: HandleId) {
        self.sender.send(AssetCommand::Destroy(id)).unwrap();
    }
}

#[derive(Resource, Debug)]
pub struct AssetManager {
    loaders: Vec<Arc<dyn Loader>>,
    extension_to_loader: FxHashMap<String, usize>,
    asset_channels: Arc<RwLock<FxHashMap<TypeId, Box<dyn AnyAssetCommandChannel>>>>,
    asset_handle_ref_channel: HandleRefChannel,
    asset_handle_ref_count: FxHashMap<HandleId, usize>,
    asset_io: Arc<dyn AssetIo>,
}

impl Default for AssetManager {
    fn default() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                Self {
                    loaders: Vec::default(),
                    extension_to_loader: FxHashMap::default(),
                    asset_channels: Arc::new(RwLock::new(FxHashMap::default())),
                    asset_handle_ref_channel: HandleRefChannel::default(),
                    asset_handle_ref_count: FxHashMap::default(),
                    asset_io: Arc::new(crate::io::WasmAssetIo::default()),
                }
            } else {
                Self {
                    loaders: Vec::default(),
                    extension_to_loader: FxHashMap::default(),
                    asset_channels: Arc::new(RwLock::new(FxHashMap::default())),
                    asset_handle_ref_channel: HandleRefChannel::default(),
                    asset_handle_ref_count: FxHashMap::default(),
                    asset_io: Arc::new(crate::io::FileAssetIo::default()),
                }
            }
        }
    }
}

impl AssetManager {
    pub fn new<T: AssetIo>(asset_io: T) -> Self {
        Self {
            loaders: Vec::default(),
            extension_to_loader: FxHashMap::default(),
            asset_channels: Arc::new(RwLock::new(FxHashMap::default())),
            asset_handle_ref_channel: HandleRefChannel::default(),
            asset_handle_ref_count: FxHashMap::default(),
            asset_io: Arc::new(asset_io),
        }
    }

    pub fn load<T: Asset, P: Into<AssetPath>>(&mut self, file_path: P) -> Handle<T> {
        let asset_path = file_path.into();
        let handle_id: HandleId = HandleId::new::<T>(&asset_path);

        let loader = self
            .find_loader(&asset_path.extension)
            .expect("Asset loader not found");

        let asset_channels = self.asset_channels.clone();

        let asset_io = self.asset_io.clone();
        crate::io::spawn(async move {
            let data = asset_io.load(&asset_path.path).await;

            let mut context = LoaderContext {
                asset: None,
                path: asset_path.path.as_path(),
            };

            loader.load(data, &mut context);

            let asset_channels = asset_channels.read().unwrap();
            let asset_channel = asset_channels.get(&handle_id.get_type()).unwrap();

            asset_channel.create(handle_id, context.asset.unwrap());
        });

        Handle::strong(handle_id, self.asset_handle_ref_channel.sender.clone())
    }

    pub fn register_asset_type<T: Asset>(&self) -> Assets<T> {
        let type_id = TypeId::of::<T>();
        self.asset_channels
            .write()
            .unwrap()
            .insert(type_id, Box::new(AssetCommandChannel::<T>::default()));

        Assets::new(self.asset_handle_ref_channel.sender.clone())
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

    fn update_asset_storage<T: Asset>(&self, assets: &mut Assets<T>) {
        let type_id = TypeId::of::<T>();
        let asset_channels = self.asset_channels.read().unwrap();
        let asset_channel = asset_channels.get(&type_id).unwrap();
        let asset_channel = asset_channel
            .downcast_ref::<AssetCommandChannel<T>>()
            .unwrap();

        loop {
            match asset_channel.receiver.try_recv() {
                Ok(AssetCommand::Create(AssetCreateCommand { id, asset })) => {
                    assets.set(&id, asset);
                }
                Ok(AssetCommand::Destroy(id)) => {
                    assets.remove(&id);
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => panic!("Asset command channel disconnected"),
            }
        }
    }

    fn update_ref_count(&mut self) {
        loop {
            match self.asset_handle_ref_channel.receiver.try_recv() {
                Ok(HandleRef::Increment(id)) => {
                    *self.asset_handle_ref_count.entry(id).or_insert(0) += 1;
                }
                Ok(HandleRef::Decrement(id)) => {
                    *self.asset_handle_ref_count.entry(id).or_insert(0) -= 1;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => panic!("Asset handle ref channel disconnected"),
            }
        }
    }

    fn destroy_unused_assets(&mut self) {
        for k in self
            .asset_handle_ref_count
            .iter()
            .filter_map(|(k, v)| if *v == 0 { Some(*k) } else { None })
            .collect::<Vec<HandleId>>()
        {
            self.asset_handle_ref_count.remove(&k);
            let asset_channels = self.asset_channels.read().unwrap();
            let asset_channel = asset_channels.get(&k.get_type()).unwrap();
            asset_channel.destroy(k);
        }
    }
}

pub fn update_asset_storage<T: Asset>(
    asset_manager: Res<AssetManager>,
    mut assets: ResMut<Assets<T>>,
) {
    asset_manager.update_asset_storage(&mut assets);
}

pub fn update_ref_count(mut asset_manager: ResMut<AssetManager>) {
    asset_manager.update_ref_count();
}

pub fn destroy_unused_assets(mut asset_manager: ResMut<AssetManager>) {
    asset_manager.destroy_unused_assets();
}

#[cfg(test)]
mod tests {
    use std::{path::Path, thread, time::Duration};

    use crate::{io::FileAssetIo, Asset, AssetManager, Assets, Handle, Loader};

    #[derive(Debug)]
    pub struct TestAsset {
        _data: Vec<u8>,
    }
    impl Asset for TestAsset {}

    #[derive(Debug)]
    pub struct TestLoader {}
    impl Loader for TestLoader {
        fn extension(&self) -> &[&str] {
            &["test"]
        }

        fn load(&self, data: Vec<u8>, context: &mut crate::LoaderContext) {
            context.set_asset(TestAsset { _data: data });
        }
    }

    fn create_dir_and_file(file: impl AsRef<Path>) -> tempfile::TempDir {
        let asset_dir = tempfile::tempdir().unwrap();
        std::fs::write(asset_dir.path().join(file), &[]).unwrap();
        asset_dir
    }

    fn setup(asset_path: impl AsRef<Path>) -> AssetManager {
        AssetManager::new(FileAssetIo::new(asset_path))
    }

    fn run_systems(asset_manager: &mut AssetManager, assets: &mut Assets<TestAsset>) {
        asset_manager.update_asset_storage(assets);
        asset_manager.update_ref_count();
        asset_manager.destroy_unused_assets();
    }

    #[test]
    fn test() {
        let dir = create_dir_and_file("file.test");
        let mut asset_manager = setup(dir.path());

        let mut assets = asset_manager.register_asset_type::<TestAsset>();
        asset_manager.register_loader(TestLoader {});

        let handle: Handle<TestAsset> = asset_manager.load("file.test");

        let timeout = 2000;
        let waiting_tick = 500;
        let mut waiting_time = 0;
        loop {
            run_systems(&mut asset_manager, &mut assets);
            let asset = assets.get(&handle.id);

            if asset.is_some() || waiting_time > timeout {
                break;
            }

            thread::sleep(Duration::from_millis(waiting_tick));
            waiting_time += waiting_tick;
        }

        let asset = assets.get(&handle.id);
        assert!(asset.is_some());
        assert_eq!(
            asset_manager.asset_handle_ref_count.get(&handle.id),
            Some(&1)
        )
    }
}

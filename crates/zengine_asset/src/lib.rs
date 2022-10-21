mod asset_manager;
mod assets;
mod handle;
mod io;
mod io_task;

use std::path::{Path, PathBuf};

pub use asset_manager::*;
pub use assets::*;
pub use handle::*;
use zengine_engine::{Engine, Module, Stage};

/// Asset Events fired when an asset has been loaded or unloaded
#[derive(Debug)]
pub enum AssetEvent<T: Asset> {
    /// Asset Loaded from disk, contains a weak [Handle] to the asset that
    /// can be used to retrieve the asset from the [Assets] storage
    Loaded(Handle<T>),
    /// Asset Unloaded from the [Assets] storage, contains a weak [Handle] to the asset.
    Unloaded(Handle<T>),
}

/// Adds assets support to the engine
///
/// Assets are typed collections, which are added as Resources.
#[derive(Default)]
pub struct AssetModule {
    asset_base_path: Option<PathBuf>,
}

impl AssetModule {
    /// Configure the [AssetModule] specifying the `asset_base_path`
    ///
    /// Note: The Android platform doesn't take in cosideration the
    /// `asset_base_path`
    pub fn new<P: AsRef<Path>>(asset_base_path: P) -> Self {
        Self {
            asset_base_path: Some(asset_base_path.as_ref().to_path_buf()),
        }
    }
}

impl Module for AssetModule {
    fn init(self, engine: &mut zengine_engine::Engine) {
        if let Some(asset_base_path) = self.asset_base_path {
            log::trace!("base asset path {:?}", asset_base_path);

            #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
            let asset_io = crate::io::FileAssetIo::new(asset_base_path);

            #[cfg(target_arch = "wasm32")]
            let asset_io = crate::io::WasmAssetIo::new(asset_base_path);

            #[cfg(target_os = "android")]
            let asset_io = crate::io::AndroidAssetIo::default();

            engine.world.create_resource(AssetManager::new(asset_io));
        } else {
            engine.world.create_resource(AssetManager::default());
        }

        engine.add_system_into_stage(update_ref_count, Stage::PostUpdate);
        engine.add_system_into_stage(destroy_unused_assets, Stage::PostUpdate);
    }
}

/// Add functionalities to add new [Asset] and new [AssetLoader] to the engine
pub trait AssetExtension {
    /// Add an [Asset] to the engine
    ///
    /// This method create the [Assets] storage for the target `Asset`
    /// register the `Asset` type to the [AssetManager]
    /// and add a system to update the `Assets` storage when and Asset
    /// is loaded or unloaded
    fn add_asset<T: Asset>(&mut self) -> &mut Self;

    /// Add an [AssetLoader] to the engine
    ///
    /// This method register an asset loader to the [AssetManager]
    fn add_asset_loader<T: AssetLoader>(&mut self, loader: T) -> &mut Self;
}

impl AssetExtension for Engine {
    fn add_asset<T: Asset>(&mut self) -> &mut Self {
        if self.world.get_resource::<Assets<T>>().is_some() {
            self
        } else {
            let assets = {
                let asset_manager = self.world.get_resource::<AssetManager>().unwrap();
                asset_manager.register_asset_type::<T>()
            };

            self.world.create_resource(assets);

            self.add_system_into_stage(update_asset_storage::<T>, Stage::PreUpdate);

            self
        }
    }

    fn add_asset_loader<T: AssetLoader>(&mut self, loader: T) -> &mut Self {
        {
            let mut asset_manager = self.world.get_mut_resource::<AssetManager>().unwrap();
            asset_manager.register_loader(loader);
        }

        self
    }
}

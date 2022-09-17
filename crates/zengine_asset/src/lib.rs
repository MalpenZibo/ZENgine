mod asset_manager;
mod assets;
mod handle;
mod io;

pub mod image_loader;

use std::path::{Path, PathBuf};

pub use asset_manager::*;
pub use assets::*;
pub use handle::*;
use zengine_engine::{Engine, Module, StageLabel};

#[derive(Default)]
pub struct AssetModule {
    asset_base_path: Option<PathBuf>,
}

impl AssetModule {
    pub fn new<P: AsRef<Path>>(asset_base_path: P) -> Self {
        Self {
            asset_base_path: Some(asset_base_path.as_ref().to_path_buf()),
        }
    }
}

impl Module for AssetModule {
    fn init(self, engine: &mut zengine_engine::Engine) {
        if let Some(asset_base_path) = self.asset_base_path {
            #[cfg(not(target_arch = "wasm32"))]
            let asset_io = crate::io::FileAssetIo::new(asset_base_path);

            #[cfg(target_arch = "wasm32")]
            let asset_io = crate::io::WasmAssetIo::new(asset_base_path);

            engine.world.create_resource(AssetManager::new(asset_io));
        } else {
            engine.world.create_resource(AssetManager::default());
        }

        engine.add_system_into_stage(destroy_unused_assets, StageLabel::PostUpdate);
        engine.add_system_into_stage(update_ref_count, StageLabel::PostUpdate);
    }
}

pub trait AssetExtension {
    fn add_asset<T: Asset>(&mut self) -> &mut Self;

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

            self.add_system_into_stage(update_asset_storage::<T>, StageLabel::PreUpdate);

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

use crossbeam_channel::Sender;
use downcast_rs::{impl_downcast, Downcast};
use rustc_hash::FxHashMap;
use std::{ffi::OsStr, path::PathBuf};
use zengine_macro::Resource;

use crate::{
    handle::{HandleId, HandleRef},
    Handle,
};

/// An Asset rappresent any kind of external data like
/// images, sound, text file etc..
///
/// To load and asset into you game you have to load from
/// the filesystem with [AssetManager::load](crate::AssetManager::load)
///
/// You should avoid to implement the Asset trait manually and use instead
/// the [asset derive macro](zengine_macro::Asset)
pub trait Asset: Downcast + Send + Sync + std::fmt::Debug + 'static {
    /// Returns an unique asset id.
    ///
    /// Normally it's used the asset path as unique id but in case
    /// this is not possible (eg: get multiple id for the same asset path)
    /// the engine will call this function.
    ///
    /// The [asset derive macro](zengine_macro::Asset) implement this function
    /// in the following way
    ///
    /// ```
    /// static ASSETTEST_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    /// impl Asset for AssetTest {
    ///    fn next_counter() -> u64
    ///    where
    ///        Self: Sized,
    ///    {
    ///        ASSETTEST_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ///    }
    ///}
    /// ```
    fn next_counter() -> u64
    where
        Self: Sized;
}
impl_downcast!(Asset);

/// Stores Assets of a given type
///
/// Each asset is mapped by an unique [`HandleId`], allowing any [`Handle`] with the same
/// [`HandleId`] to access it.
/// One asset remain loaded as long as a Strong handle to that asset exists.
///
/// To get a reference to an asset without forcing it to stay loadid you can use a Weak handle
#[derive(Resource, Debug)]
pub struct Assets<T: Asset> {
    assets: FxHashMap<HandleId, T>,
    pub(crate) sender: Sender<HandleRef>,
}

impl<T: Asset> Assets<T> {
    pub(crate) fn new(sender: Sender<HandleRef>) -> Self {
        Self {
            assets: FxHashMap::default(),
            sender,
        }
    }

    /// Checks if an asset exists for the given handle
    pub fn contains(&self, handle: &Handle<T>) -> bool {
        self.assets.contains_key(&handle.id)
    }

    /// Get an asset reference for the given handle
    pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
        self.assets.get(&handle.id)
    }

    /// Get a mutable asset reference for the given handle
    pub fn get_mut(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        self.assets.get_mut(&handle.id)
    }

    /// Gets an iterator over the assets in the storage
    pub fn iter(&self) -> impl Iterator<Item = (&HandleId, &T)> {
        self.assets.iter()
    }

    /// Gets a mutable iterator over the assets in the storage
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&HandleId, &mut T)> {
        self.assets.iter_mut()
    }

    /// Add an asset to the storage returning a strong handle to that asset
    pub fn add(&mut self, asset: T) -> Handle<T> {
        let handle = Handle::strong(
            HandleId::new_from_u64::<T>(T::next_counter()),
            self.sender.clone(),
        );

        self.set_untracked(handle.id, asset);

        handle
    }

    /// Add/Replace the asset pointed by the given handle
    /// returning a strong handle to that asset
    pub fn set(&mut self, handle: Handle<T>, asset: T) -> Handle<T> {
        let id = handle.id;
        self.set_untracked(id, asset);

        Handle::strong(id, self.sender.clone())
    }

    /// Add/Replace the asset pointed by the given handle
    pub fn set_untracked(&mut self, handle_id: HandleId, asset: T) {
        self.assets.insert(handle_id, asset);
    }

    /// Remove the asset pointed by the given handle from the storage
    ///
    /// The asset is returned
    pub fn remove<H: Into<HandleId>>(&mut self, handle: H) -> Option<T> {
        let id = handle.into();
        self.assets.remove(&id)
    }

    /// Gets the number of assets in the storage
    pub fn len(&self) -> usize {
        self.assets.len()
    }

    /// Returns `true` if there are no stored assets
    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }
}

/// Represents a path to an asset in the file system
#[derive(Debug)]
pub struct AssetPath {
    pub(crate) path: PathBuf,
    pub(crate) extension: String,
}

impl From<&str> for AssetPath {
    fn from(file_path: &str) -> Self {
        let path = std::path::Path::new(file_path);

        let extension = path
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or("")
            .to_owned();

        AssetPath {
            path: path.into(),
            extension,
        }
    }
}

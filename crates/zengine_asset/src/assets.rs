use crossbeam_channel::Sender;
use downcast_rs::{impl_downcast, Downcast};
use rustc_hash::FxHashMap;
use std::{ffi::OsStr, path::PathBuf};
use zengine_macro::Resource;

use crate::{
    handle::{HandleId, HandleRef},
    Handle,
};

pub trait Asset: Downcast + Send + Sync + std::fmt::Debug + 'static {}
impl_downcast!(Asset);

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

    pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
        self.assets.get(&handle.id)
    }

    pub fn get_mut(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        self.assets.get_mut(&handle.id)
    }

    pub fn set(&mut self, handle: Handle<T>, asset: T) -> Handle<T> {
        let id = handle.id;
        self.set_untracked(id, asset);

        Handle::strong(id, self.sender.clone())
    }

    pub fn set_untracked(&mut self, handle_id: HandleId, asset: T) {
        self.assets.insert(handle_id, asset);
    }

    pub fn remove<H: Into<HandleId>>(&mut self, handle: H) -> Option<T> {
        let id = handle.into();
        self.assets.remove(&id)
    }

    pub fn len(&self) -> usize {
        self.assets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }
}

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

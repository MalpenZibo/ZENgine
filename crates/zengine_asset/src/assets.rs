use crossbeam_channel::Sender;
use downcast_rs::{impl_downcast, Downcast};
use rustc_hash::FxHashMap;
use std::{ffi::OsStr, path::PathBuf};

use crate::handle::{HandleId, HandleRef};

pub trait Asset: Downcast + Send + Sync + std::fmt::Debug + 'static {}
impl_downcast!(Asset);

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

    pub fn get(&self, id: &HandleId) -> Option<&T> {
        self.assets.get(id)
    }

    pub fn set(&mut self, id: &HandleId, asset: T) {
        self.assets.insert(*id, asset);
    }

    pub fn remove(&mut self, id: &HandleId) -> Option<T> {
        self.assets.remove(id)
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

use downcast_rs::{impl_downcast, Downcast};
use std::{ffi::OsStr, path::PathBuf};

pub trait Asset: Downcast + Send + Sync + std::fmt::Debug + 'static {}
impl_downcast!(Asset);

pub struct Assets<T: Asset> {
    assets: Vec<T>,
}

#[derive(Debug)]
pub struct AssetPath {
    pub(crate) path: PathBuf,
    pub(crate) extension: String,
}

impl From<&str> for AssetPath {
    fn from(file_path: &str) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {

                let path = std::path::Path::new("assets/");
                let path = path.join(&file_path);

                let extension = path.as_path().extension().and_then(OsStr::to_str).unwrap_or("").to_owned();

                AssetPath { path, extension }
            } else {
                let mut path =
                    std::env::current_exe().unwrap_or_else(|e| panic!("current exe path error: {}", e));

                path.pop();

                path.push("assets");
                path.push(&file_path);

                let extension = path.as_path().extension().and_then(OsStr::to_str).unwrap_or_else(|| "").to_owned();

                AssetPath { path, extension }
            }
        }
    }
}

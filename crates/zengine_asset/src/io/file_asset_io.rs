use std::{
    env,
    path::{Path, PathBuf},
};

use async_trait::async_trait;

use super::AssetIo;

#[derive(Debug)]
pub struct FileAssetIo {
    base_path: PathBuf,
}

impl Default for FileAssetIo {
    fn default() -> Self {
        Self {
            base_path: Self::get_base_path(),
        }
    }
}

impl FileAssetIo {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: Self::get_base_path().join(base_path.as_ref()),
        }
    }

    pub fn get_base_path() -> PathBuf {
        if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            PathBuf::from(manifest_dir)
        } else {
            env::current_exe()
                .map(|path| {
                    path.parent()
                        .map(|exe_parent_path| exe_parent_path.to_owned())
                        .unwrap()
                })
                .unwrap()
        }
    }
}

#[async_trait]
impl AssetIo for FileAssetIo {
    async fn load(&self, asset_path: &Path) -> Vec<u8> {
        let full_path = self.base_path.join(asset_path);
        let full_path = full_path.as_path();
        let data = async move {
            std::fs::read(full_path)
                .unwrap_or_else(|e| panic!("Could not load file {:?}: {}", &full_path, e))
        };

        data.await
    }
}

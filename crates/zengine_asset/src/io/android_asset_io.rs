use std::{ffi::CString, path::Path};

use async_trait::async_trait;

use super::AssetIo;

#[derive(Default, Debug)]
pub struct AndroidAssetIo;

#[async_trait]
impl AssetIo for AndroidAssetIo {
    async fn load(&self, asset_path: &Path) -> Vec<u8> {
        let full_path = asset_path;

        let asset_manager = ndk_glue::native_activity().asset_manager();
        let mut opened_asset = asset_manager
            .open(&CString::new(full_path.to_str().unwrap()).unwrap())
            .unwrap();

        let bytes = opened_asset.get_buffer().unwrap();
        bytes.to_vec()
    }
}

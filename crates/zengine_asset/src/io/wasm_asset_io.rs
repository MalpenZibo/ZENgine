use async_trait::async_trait;
use js_sys::Uint8Array;
use std::path::{Path, PathBuf};
use wasm_bindgen::JsCast;

use super::AssetIo;

#[derive(Default, Debug)]
pub struct WasmAssetIo {
    base_path: PathBuf,
}

impl WasmAssetIo {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
}

#[async_trait(?Send)]
impl AssetIo for WasmAssetIo {
    async fn load(&self, asset_path: &Path) -> Vec<u8> {
        let full_path = self.base_path.join(asset_path);
        let full_path = full_path.as_path();

        let window = web_sys::window().unwrap();

        let resp_value = wasm_bindgen_futures::JsFuture::from(
            window.fetch_with_str(full_path.to_str().unwrap()),
        )
        .await
        .unwrap();

        let resp: web_sys::Response = resp_value.dyn_into().unwrap();
        let resp_value = wasm_bindgen_futures::JsFuture::from(resp.array_buffer().unwrap())
            .await
            .unwrap();

        Uint8Array::new(&resp_value).to_vec()
    }
}

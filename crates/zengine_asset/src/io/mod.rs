use async_trait::async_trait;
use std::future::Future;
use std::path::Path;

#[cfg(not(target_arch = "wasm32"))]
mod file_asset_io;

#[cfg(target_arch = "wasm32")]
mod wasm_asset_io;

#[cfg(target_arch = "wasm32")]
pub(crate) fn spawn<F: Future<Output = ()> + 'static>(future: F) {
    wasm_bindgen_futures::spawn_local(future);
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn spawn<F: Future<Output = ()> + Send + 'static>(future: F) {
    std::thread::spawn(|| pollster::block_on(future));
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait AssetIo: std::fmt::Debug + Send + Sync + 'static {
    async fn load(&self, asset_path: &Path) -> Vec<u8>;
}

#[cfg(not(target_arch = "wasm32"))]
pub use file_asset_io::FileAssetIo;

#[cfg(target_arch = "wasm32")]
pub use wasm_asset_io::WasmAssetIo;

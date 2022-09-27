use async_trait::async_trait;
use std::path::Path;

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
mod file_asset_io;

#[cfg(target_arch = "wasm32")]
mod wasm_asset_io;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait AssetIo: std::fmt::Debug + Send + Sync + 'static {
    async fn load(&self, asset_path: &Path) -> Vec<u8>;
}

#[cfg(target_os = "android")]
mod android_asset_io;

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub use file_asset_io::FileAssetIo;

#[cfg(target_arch = "wasm32")]
pub use wasm_asset_io::WasmAssetIo;

#[cfg(target_os = "android")]
pub use android_asset_io::AndroidAssetIo;

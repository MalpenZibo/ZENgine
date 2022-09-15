mod asset_manager;
mod assets;
mod handle;
mod io_task;

#[cfg(not(target_arch = "wasm32"))]
mod file_asset_io;

#[cfg(target_arch = "wasm32")]
mod wasm_asset_io;

pub mod audio_loader;
pub mod image_loader;

pub use asset_manager::*;
pub use assets::*;
pub use handle::*;

use crossbeam_channel::Receiver;
use rustc_hash::FxHashMap;

use std::{
    any::TypeId,
    collections::VecDeque,
    ffi::OsStr,
    marker::PhantomData,
    path::{Path, PathBuf},
};
use zengine_ecs::system::ResMut;
use zengine_macro::Resource;

pub mod asset_manager;
pub mod assets;
pub mod handle;
mod io_task;

#[cfg(not(target_arch = "wasm32"))]
mod file_asset_io;

#[cfg(target_arch = "wasm32")]
mod wasm_asset_io;

pub mod audio_loader;
pub mod image_loader;

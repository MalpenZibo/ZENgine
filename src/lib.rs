pub use zengine_engine::*;

pub mod ecs {
    pub use zengine_ecs::*;
}

pub mod core {
    pub use zengine_core::*;
}

pub mod math {
    pub use glam::*;
}

pub mod graphic {
    pub use zengine_graphic::*;
}

pub mod input {
    pub use zengine_input::*;
}

pub mod physics {
    pub use zengine_physics::*;
}

pub mod window {
    pub use zengine_window::*;
}

pub mod audio {
    pub use zengine_audio::*;
}

pub mod asset {
    pub use zengine_asset::*;
}

extern crate zengine_macro;
pub use zengine_macro::*;

#[cfg(target_os = "android")]
pub use ndk_glue;

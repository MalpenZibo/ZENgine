#![doc(
    html_logo_url = "https://raw.githubusercontent.com/MalpenZibo/ZENgine/main/assets/branding/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/MalpenZibo/ZENgine/main/assets/branding/logo.svg"
)]

//! [<img src="https://raw.githubusercontent.com/MalpenZibo/ZENgine/main/assets/branding/logo_extended.svg" height="200" />](https://malpenzibo.github.io/ZENgine/)
//!
//! ZENgine is a simple open-source modular game engine built in Rust for didactical purpose
//!
//! ## Example
//! Here is a simple "Hello World" ZENgine app:
//! ```
//! use zengine::Engine;
//!
//! fn main() {
//!    Engine::default()
//!        .add_system(hello_world_system)
//!        .run();
//! }
//!
//! fn hello_world_system() {
//!    println!("hello world");
//! }
//! ```
//!
//! ### This Crate
//! The `zengine` crate is a container crate that makes it easier to consume ZENgine subcrates.
//!
//! If you prefer, you can also consume the individual ZENgine crates directly.
//! Each module in the root of this crate, can be found on crates.io
//! with `zengine_` appended to the front, e.g. `engine` -> [`zengine_engine`](https://docs.rs/zengine_engine/*/zengine_engine/).

pub use zengine_engine::*;

pub mod ecs {
    //! Entity-component-system.
    pub use zengine_ecs::*;
}

pub mod core {
    //! Core functionalities.
    pub use zengine_core::*;
}

pub mod math {
    //! Math types.
    pub use glam::*;
}

pub mod graphic {
    //! Graphics functionlities. eg: Camera, Texture, Render, Sprite.
    pub use zengine_graphic::*;
}

pub mod input {
    //! Resources and events for inputs
    pub use zengine_input::*;
}

pub mod physics {
    //! Collision system and shapes
    pub use zengine_physics::*;
}

pub mod window {
    //! Creation, configuration and management of the main window.
    pub use zengine_window::*;
}

pub mod audio {
    //! Provides types and module for audio playback.
    pub use zengine_audio::*;
}

pub mod asset {
    //! Load and store Assets for the Engine.
    pub use zengine_asset::*;
}

pub mod gamepad {
    //! ZENgine interface with `GilRs` - "Game Input Library for Rust" - to handle gamepad inputs.
    pub use zengine_gamepad::*;
}

extern crate zengine_macro;
pub use zengine_macro::*;

#[cfg(target_os = "android")]
pub use ndk_glue;

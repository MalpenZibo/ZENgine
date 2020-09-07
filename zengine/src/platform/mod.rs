extern crate sdl2;

use crate::core::Resource;
use sdl2::VideoSubsystem;

mod platform_system;
pub use self::platform_system::PlatformSystem;

impl Resource for VideoSubsystem {}

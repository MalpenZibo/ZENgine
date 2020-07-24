extern crate sdl2;

use crate::core::Resource;
use sdl2::VideoSubsystem;

mod system;
pub use self::system::PlatformSystem;

impl Resource for VideoSubsystem {}

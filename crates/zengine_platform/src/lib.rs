extern crate sdl2;

use std::fmt::Debug;

use sdl2::VideoSubsystem;
use zengine_ecs::world::UnsendableResource;

pub mod device;
mod platform_system;
pub use self::platform_system::*;

pub struct VideoSubsystemWrapper(pub VideoSubsystem);
impl Debug for VideoSubsystemWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
impl UnsendableResource for VideoSubsystemWrapper {}

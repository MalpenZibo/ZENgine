extern crate sdl2;

use crate::core::Resource;
use sdl2::Sdl;

pub struct Platform {
    pub context: Sdl,
}
impl Resource for Platform {}

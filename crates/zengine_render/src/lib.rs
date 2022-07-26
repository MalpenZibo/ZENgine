use zengine_graphic::{Color, SpriteType};
use zengine_macro::{Component, Resource};
use zengine_math::Vector3;

mod gl_utilities;
mod render_system;

pub use render_system::{render_system, setup_render};

#[derive(Copy, Clone)]
pub enum CollisionTrace {
    Active,
    Inactive,
}

#[derive(Clone)]
pub struct WindowSpecs {
    title: String,
    width: u32,
    height: u32,
    fullscreen: bool,
}

impl WindowSpecs {
    pub fn new(title: String, width: u32, height: u32, fullscreen: bool) -> Self {
        WindowSpecs {
            title,
            width,
            height,
            fullscreen,
        }
    }
}

impl Default for WindowSpecs {
    fn default() -> Self {
        WindowSpecs {
            title: String::from("zengine"),
            width: 800,
            height: 600,
            fullscreen: false,
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct Background {
    pub color: Color,
}

#[derive(Component, Debug)]
pub struct Sprite<ST: SpriteType> {
    pub width: f32,
    pub height: f32,
    pub origin: Vector3,
    pub color: Color,
    pub sprite_type: ST,
}

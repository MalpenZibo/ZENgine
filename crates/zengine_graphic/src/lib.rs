use zengine_engine::{Module, StageLabel};
use zengine_macro::{Component, Resource};

mod camera;
mod color;
mod renderer;
pub(crate) mod renderer_utils;
mod texture;

#[derive(Resource, Debug, Default)]
pub struct Background {
    pub color: Color,
}

#[derive(Component, Debug)]
pub struct Sprite<ST: SpriteType> {
    pub width: f32,
    pub height: f32,
    pub origin: glam::Vec3,
    pub color: Color,
    pub sprite_type: ST,
}

#[derive(Debug)]
pub struct RenderModule<ST: SpriteType> {
    _phantom: std::marker::PhantomData<ST>,
}

impl<ST: SpriteType> Default for RenderModule<ST> {
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData::default(),
        }
    }
}

impl<ST: SpriteType> Module for RenderModule<ST> {
    fn init(self, engine: &mut zengine_engine::Engine) {
        engine
            .add_startup_system(setup_render)
            .add_system(texture_loader::<ST>)
            .add_system_into_stage(renderer::<ST>, StageLabel::Render);
    }
}

pub use camera::*;
pub use color::*;
pub use renderer::*;
pub use texture::*;

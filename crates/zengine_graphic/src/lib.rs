use sprite::{setup_sprite_render, sprite_render};
use zengine_asset::AssetExtension;
use zengine_engine::{schedule::default_schedule::{PostRender, PreRender, Render}, Module};
use zengine_macro::Resource;

mod camera;
mod color;
mod image_asset;
mod renderer;
mod sprite;
mod texture;
mod texture_atlas;
mod vertex;

pub use camera::*;
pub use color::*;
pub use image_asset::*;
pub use renderer::*;
pub use sprite::*;
pub use texture::*;
pub use texture_atlas::*;
pub use vertex::*;

/// [Resource](zengine_ecs::Resource) that describe the color used
/// to clear of the view
#[derive(Resource, Debug, Default)]
pub struct Background {
    pub color: Color,
}

/// Adds graphic support to the engine using a wgpu based renderer
#[derive(Default, Debug)]
pub struct GraphicModule;

impl Module for GraphicModule {
    fn init(self, engine: &mut zengine_engine::Engine) {
        engine
            .add_asset::<Image>()
            .add_asset_loader(ImageLoader)
            .add_asset::<Texture>()
            .add_asset::<TextureAtlas>()
            .add_startup_system(setup_render)
            .add_startup_system(setup_camera)
            .add_startup_system(setup_sprite_render)
            .add_system_into_schedule(prepare_texture_asset, PreRender)
            .add_system_into_schedule(prepare_texture_atlas_asset, PreRender)
            .add_system_into_schedule(clear, PreRender)
            .add_system_into_schedule(camera_render, Render)
            .add_system_into_schedule(sprite_render(), Render)
            .add_system_into_schedule(present, PostRender);
    }
}

use sprite::{setup_sprite_render, sprite_render};
use zengine_asset::AssetExtension;
use zengine_engine::{Module, Stage};
use zengine_macro::Resource;

mod camera;
mod color;
mod image_asset;
mod renderer;
mod sprite;
mod texture;
mod texture_atlas;
pub(crate) mod vertex;

pub use camera::*;
pub use color::*;
pub use image_asset::*;
pub use renderer::*;
pub use sprite::*;
pub use texture::*;
pub use texture_atlas::*;

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
            .add_system_into_stage(prepare_texture_asset, Stage::Render)
            .add_system_into_stage(prepare_texture_atlas_asset, Stage::Render)
            .add_system_into_stage(clear, Stage::Render)
            .add_system_into_stage(camera_render, Stage::Render)
            .add_system_into_stage(sprite_render, Stage::Render)
            .add_system_into_stage(present, Stage::Render);
    }
}

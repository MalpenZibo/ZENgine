use sprite::{setup_sprite_render, sprite_render};
use zengine_asset::AssetExtension;
use zengine_engine::{Module, StageLabel};
use zengine_macro::Resource;

mod camera;
mod color;
mod image_asset;
mod renderer;
mod sprite;
mod texture_atlas;
pub(crate) mod vertex;

pub use camera::*;
pub use color::*;
pub use image_asset::*;
pub use renderer::*;
pub use sprite::*;
pub use texture_atlas::*;

#[derive(Resource, Debug, Default)]
pub struct Background {
    pub color: Color,
}

#[derive(Default, Debug)]
pub struct RenderModule;

impl Module for RenderModule {
    fn init(self, engine: &mut zengine_engine::Engine) {
        engine
            .add_asset::<Image>()
            .add_asset_loader(ImageLoader)
            .add_startup_system(setup_render)
            .add_startup_system(setup_camera)
            .add_startup_system(setup_sprite_render)
            .add_system_into_stage(prepare_image_asset, StageLabel::Render)
            .add_system_into_stage(clear, StageLabel::Render)
            .add_system_into_stage(camera_render, StageLabel::Render)
            .add_system_into_stage(sprite_render, StageLabel::Render)
            .add_system_into_stage(present, StageLabel::Render);
    }
}

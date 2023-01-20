use std::{ops::MulAssign, process::Command};

use ::glyph_brush::Section;
use ab_glyph::FontArc;
use glam::{UVec2, Vec2};
use glyph_brush_layout::FontId;
use glyph_brush_test::GlyphBrush;
use log::info;

use text_render::TextRender;
use zengine_asset::{AssetEvent, AssetExtension, Assets};
use zengine_core::Transform;
use zengine_ecs::{
    query::{Query, QueryIter},
    system::{Commands, EventStream, Local, Res, ResMut},
    Entity,
};
use zengine_engine::{Engine, Module, Stage};
use zengine_window::WindowSpecs;

mod builder;
mod cache;
mod font;
mod glyph_brush_test;
mod pipeline;
mod text;
mod text_render;

pub use font::*;
pub use text::*;
use zengine_graphic::{
    ActiveCamera, Camera, CameraBuffer, CameraMode, CameraUniform, Device, RenderContextInstance,
    Surface, UsedCamera,
};
use zengine_macro::Resource;

// use crate::builder::GlyphBrushBuilder;

/// A region of the screen.
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

///A [Module] that defines an interface for windowing support in ZENgine.
#[derive(Default, Debug)]
pub struct TextModule;
impl Module for TextModule {
    fn init(self, engine: &mut Engine) {
        engine
            .add_asset::<Font>()
            .add_asset_loader(FontLoader)
            .add_system_into_stage(font_asset_event_handler, Stage::PreUpdate)
            .add_system_into_stage(text_render, Stage::Render);
    }
}

fn font_asset_event_handler(
    mut commands: Commands,
    font_asset_event: EventStream<AssetEvent<Font>>,
    fonts: Option<ResMut<Assets<Font>>>,
    device: Option<Res<Device>>,
    surface: Res<Surface>,
    camera_buffer: Option<Res<CameraBuffer>>,
) {
    if let (Some(device), Some(surface_config), Some(camera_buffer), Some(mut fonts)) =
        (device, surface.get_config(), camera_buffer, fonts)
    {
        if font_asset_event.read().count() > 0 {
            let text_render =
                TextRender::new(&device, surface_config.format, &camera_buffer, &mut fonts);
            commands.create_resource(text_render);

            // let mut font_iter = fonts.iter_mut();
            // if let Some((_, font)) = font_iter.next() {
            //     let mut builder = GlyphBrushBuilder::using_font(font.font.clone());

            //     font.font_id = Some(FontId(0));

            //     for (_, font) in font_iter {
            //         let id = builder.add_font(font.font.clone());
            //         font.font_id = Some(id);
            //     }
            //     info!("glyph_brush created ");
            //     glyph_brush
            //         .0
            //         .replace(builder.build(&device, surface_config.format));
            // } else {
            //     glyph_brush.0.take();
            // }
        }
    }
}

// #[derive(Resource, Default, Debug)]
// pub struct GlyphBrush2(Option<GlyphBrush<()>>);

fn text_render(
    mut text_render: Option<ResMut<TextRender>>,
    texts: Query<(&Text, &Transform)>,
    fonts: Option<Res<Assets<Font>>>,
    device: Option<Res<Device>>,
    mut render_context: ResMut<RenderContextInstance>,
    staging_belt: Local<Option<wgpu::util::StagingBelt>>,
    camera_buffer: Option<ResMut<CameraBuffer>>,
    window_specs: Res<WindowSpecs>,
    used_camera: Res<UsedCamera>,
) {
    let staging_belt = staging_belt.get_or_insert(wgpu::util::StagingBelt::new(1024));
    staging_belt.recall();

    if let (Some(mut text_render), Some(fonts), Some(device), Some(render_context)) =
        (text_render, fonts, device, render_context.as_mut())
    {
        for (text, transform) in texts.iter() {
            text_render.queue(Section {
                screen_position: (transform.position.x, transform.position.y),
                bounds: text.bounds.into(),
                text: text
                    .sections
                    .iter()
                    .map(|s| {
                        glyph_brush_test::Text::new(&s.value)
                            .with_color(s.style.color.to_array())
                            .with_scale(s.style.font_size)
                            .with_font_id(fonts.get(&s.style.font).unwrap().font_id.unwrap())
                    })
                    .collect(),
                ..Default::default()
            });

            if let Some(camera_buffer) = camera_buffer.as_ref() {
                // glyph_brush
                //     .draw_queued_with_transform(
                //         &device,
                //         staging_belt,
                //         &mut render_context.command_encoder,
                //         &render_context.texture_view,
                //         camera_buffer
                //             .camera_uniform
                //             .0
                //             .mul_mat4(&transform.get_transformation_matrix())
                //             .to_cols_array(),
                //     )
                //     .expect("Draw queued");
                // info!("here");
                text_render
                    .draw_queued(
                        &device,
                        staging_belt,
                        &mut render_context.command_encoder,
                        &render_context.texture_view,
                        100,
                        100,
                        camera_buffer,
                        &(window_specs.size.as_vec2()
                            / used_camera
                                .0
                                .as_ref()
                                .map(|c| c.0)
                                .unwrap_or_else(|| Vec2::ONE)),
                    )
                    .expect("Draw queued");
            }

            staging_belt.finish();
        }
    }
}

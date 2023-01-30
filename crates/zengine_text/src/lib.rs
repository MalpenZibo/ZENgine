use etagere::AllocId;
use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use glam::Vec2;

use log::info;
use text_atlas::TextAtlas;
use text_render::TextRenderer;
use wgpu::{Buffer, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor};
use zengine_asset::{AssetEvent, AssetExtension, Assets};
use zengine_core::Transform;
use zengine_ecs::{
    query::{Query, QueryIter},
    system::{Commands, EventStream, Local, Res, ResMut},
};
use zengine_engine::{Engine, Module, Stage};
use zengine_graphic::{
    Adapter, CameraBuffer, Device, Queue, RenderContextInstance, Surface, UsedCamera,
};
use zengine_macro::Resource;
use zengine_window::WindowSpecs;

mod error;
mod font;
mod recently_used;
mod text;
mod text_atlas;
mod text_render;

pub use font::*;
pub use text::*;

/// The color to use when rendering text.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Color {
    /// The red component of the color.
    pub r: u8,
    /// The green component of the color.
    pub g: u8,
    /// The blue component of the color.
    pub b: u8,
    /// The alpha component of the color.
    pub a: u8,
}

/// Allows text to be colored during rendering.
pub trait HasColor: Copy {
    /// The color to use when rendering text.
    fn color(&self) -> Color;
}

pub(crate) enum GpuCache {
    InAtlas { x: f32, y: f32 },
    SkipRasterization,
}

pub(crate) struct GlyphDetails {
    width: f32,
    height: f32,
    gpu_cache: GpuCache,
    atlas_id: Option<AllocId>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct GlyphToRender {
    pos: [f32; 2],
    dim: [f32; 2],
    uv: [f32; 2],
    color: [u8; 4],
}

/// The screen resolution to use when rendering text.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Resolution {
    /// The width of the screen in pixels.
    pub width: u32,
    /// The height of the screen in pixels.
    pub height: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Params {
    screen_resolution: Resolution,
    _pad: [u32; 2],
}

/// Controls the overflow behavior of any glyphs that are outside of the layout bounds.
pub enum TextOverflow {
    /// Glyphs can overflow the bounds.
    Overflow,
    /// Hide any glyphs outside the bounds. If a glyph is partially outside the bounds, it will be
    /// clipped to the bounds.
    Hide,
}

///A [Module] that defines an interface for windowing support in ZENgine.
#[derive(Default, Debug)]
pub struct TextModule;
impl Module for TextModule {
    fn init(self, engine: &mut Engine) {
        engine
            .add_asset::<Font>()
            .add_asset_loader(FontLoader)
            .add_startup_system(setup_text_render)
            .add_system_into_stage(text_render, Stage::Render);
    }
}

fn setup_text_render(
    mut commands: Commands,
    device: Option<Res<Device>>,
    queue: Option<Res<Queue>>,
    surface: Res<Surface>,
    camera_buffer: Option<Res<CameraBuffer>>,
) {
    if let (Some(device), Some(queue), Some(config), Some(camera_buffer)) =
        (device, queue, surface.get_config(), camera_buffer)
    {
        commands.create_resource(TextAtlas::new(
            &device,
            &queue,
            config.format,
            &camera_buffer,
        ));
        commands.create_resource(TextRenderer::new(&device, &queue));
    }
}

#[derive(Clone, Copy)]
struct GlyphUserData;

impl HasColor for GlyphUserData {
    fn color(&self) -> Color {
        Color {
            r: 255,
            g: 255,
            b: 0,
            a: 255,
        }
    }
}

fn text_render(
    text_renderer: Option<ResMut<TextRenderer>>,
    text_atlas: Option<ResMut<TextAtlas>>,
    texts: Query<(&Text, &Transform)>,
    fonts: Option<Res<Assets<Font>>>,
    device: Option<Res<Device>>,
    queue: Option<Res<Queue>>,
    mut render_context: ResMut<RenderContextInstance>,
    staging_belt: Local<Option<wgpu::util::StagingBelt>>,
    camera_buffer: Option<ResMut<CameraBuffer>>,
    window_specs: Res<WindowSpecs>,
    used_camera: Res<UsedCamera>,
) {
    if let (
        Some(mut text_renderer),
        Some(mut atlas),
        Some(fonts),
        Some(device),
        Some(queue),
        Some(render_context),
        Some(camera_buffer),
    ) = (
        text_renderer,
        text_atlas,
        fonts,
        device,
        queue,
        render_context.as_mut(),
        camera_buffer,
    ) {
        let mut layout1 = Layout::new(CoordinateSystem::PositiveYDown);
        layout1.reset(&LayoutSettings {
            x: 0.0,
            y: 0.0,
            ..LayoutSettings::default()
        });

        let fonts: Vec<&fontdue::Font> = fonts.iter().map(|f| &f.1.font).collect();

        let scale = window_specs.size / used_camera.get_size().unwrap_or_else(|| Vec2::ONE);

        if fonts.len() > 0 {
            layout1.append(
                fonts.as_slice(),
                &TextStyle::with_user_data("P", 100.0, 0, GlyphUserData),
            );

            text_renderer
                .prepare(
                    &device,
                    &queue,
                    &mut atlas,
                    Resolution {
                        width: 1280,
                        height: 720,
                    },
                    &fonts,
                    &[(layout1, TextOverflow::Hide)],
                    &Vec2::ONE,
                )
                .unwrap();

            let mut pass =
                &mut render_context
                    .command_encoder
                    .begin_render_pass(&RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &mut render_context.texture_view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(wgpu::Color::BLACK),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });

            text_renderer
                .render(&atlas, &mut pass, &camera_buffer)
                .unwrap();
        }
    }
}

use etagere::AllocId;
use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings};
use glam::{Mat4, Vec2};
use text_atlas::TextAtlas;
use text_render::TextRenderer;
use wgpu::{LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor};
use zengine_asset::{AssetExtension, Assets};
use zengine_core::Transform;
use zengine_ecs::{
    query::{Query, QueryIter},
    system::{Commands, Local, Res, ResMut},
};
use zengine_engine::{Engine, Module, Stage};
use zengine_graphic::{
    CameraBuffer, Color, Device, Queue, RenderContextInstance, Surface, UsedCamera,
};
use zengine_window::WindowSpecs;

mod error;
mod font;
mod recently_used;
mod text;
mod text_atlas;
mod text_render;

pub use font::*;
pub use text::*;

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

#[derive(Default, Clone, Copy)]
struct GlyphUserData {
    pub transform_matrix: Mat4,
    pub color: Color,
}

#[allow(clippy::too_many_arguments)]
fn text_render(
    text_renderer: Option<ResMut<TextRenderer>>,
    text_atlas: Option<ResMut<TextAtlas>>,
    texts: Query<(&Text, &Transform)>,
    fonts: Option<Res<Assets<Font>>>,
    device: Option<Res<Device>>,
    queue: Option<Res<Queue>>,
    mut render_context: ResMut<RenderContextInstance>,
    camera_buffer: Option<ResMut<CameraBuffer>>,
    window_specs: Res<WindowSpecs>,
    used_camera: Res<UsedCamera>,
    layouts: Local<Vec<Layout<GlyphUserData>>>,
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
        if !fonts.is_empty() {
            let scale = window_specs.size.as_vec2() / used_camera.get_size().unwrap_or(Vec2::ONE);

            let fontdue_fonts: Vec<&fontdue::Font> = fonts.iter().map(|f| &f.1 .0).collect();
            layouts.clear();

            for (text, transform) in texts.iter() {
                let transform_matrix = transform.get_transformation_matrix();
                let default_font = fonts
                    .keys()
                    .enumerate()
                    .find_map(|(index, key)| {
                        if *key == text.style.font.get_id() {
                            Some(index)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0);

                let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
                layout.reset(&LayoutSettings {
                    x: 0.0,
                    y: 0.0,
                    max_width: text.bounds.map(|b| b.x * scale.x),
                    max_height: text.bounds.map(|b| b.y * scale.y),
                    ..LayoutSettings::default()
                });

                for s in &text.sections {
                    let (font, font_size, color) = s
                        .style
                        .as_ref()
                        .map(|style| {
                            (
                                fonts
                                    .keys()
                                    .enumerate()
                                    .find_map(|(index, key)| {
                                        if *key == style.font.get_id() {
                                            Some(index)
                                        } else {
                                            None
                                        }
                                    })
                                    .unwrap_or(default_font),
                                style.font_size,
                                style.color,
                            )
                        })
                        .unwrap_or((default_font, text.style.font_size, text.style.color));

                    layout.append(
                        fontdue_fonts.as_slice(),
                        &fontdue::layout::TextStyle::with_user_data(
                            &s.value,
                            font_size,
                            font,
                            GlyphUserData {
                                color,
                                transform_matrix,
                            },
                        ),
                    );
                }

                layouts.push(layout);
            }

            text_renderer
                .prepare(&device, &queue, &mut atlas, &fontdue_fonts, layouts, &scale)
                .unwrap();

            let pass =
                &mut render_context
                    .command_encoder
                    .begin_render_pass(&RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &mut render_context.texture_view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        ..Default::default()
                    });

            text_renderer.render(&atlas, pass, &camera_buffer).unwrap();
        }
    }
}

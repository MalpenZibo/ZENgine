use glyph_brush_layout::FontId;
use log::info;
use wgpu_glyph::{GlyphBrushBuilder, Section};
use zengine_asset::{AssetEvent, AssetExtension, Assets};
use zengine_core::Transform;
use zengine_ecs::{
    query::{Query, QueryIter},
    system::{EventStream, Local, Res, ResMut},
};
use zengine_engine::{Engine, Module, Stage};
use zengine_window::WindowSpecs;

mod font;
mod text;

pub use font::*;
pub use text::*;
use zengine_graphic::{Device, RenderContextInstance, Surface};
use zengine_macro::Resource;

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
    mut glyph_brush: ResMut<GlyphBrush>,
    font_asset_event: EventStream<AssetEvent<Font>>,
    fonts: Option<ResMut<Assets<Font>>>,
    device: Option<Res<Device>>,
    surface: Res<Surface>,
) {
    if let (Some(device), Some(surface_config), Some(mut fonts)) =
        (device, surface.get_config(), fonts)
    {
        if font_asset_event.read().count() > 0 {
            let mut font_iter = fonts.iter_mut();
            if let Some((_, font)) = font_iter.next() {
                let mut builder = GlyphBrushBuilder::using_font(font.font.clone());

                font.font_id = Some(FontId(0));

                for (_, font) in font_iter {
                    let id = builder.add_font(font.font.clone());
                    font.font_id = Some(id);
                }
                info!("glyph_brush created ");
                glyph_brush
                    .0
                    .replace(builder.build(&device, surface_config.format));
            } else {
                glyph_brush.0.take();
            }
        }
    }
}

#[derive(Resource, Default, Debug)]
pub struct GlyphBrush(Option<wgpu_glyph::GlyphBrush<()>>);

fn text_render(
    mut glyph_brush: ResMut<GlyphBrush>,
    texts: Query<(&Text, &Transform)>,
    fonts: Option<Res<Assets<Font>>>,
    device: Option<Res<Device>>,
    mut render_context: ResMut<RenderContextInstance>,
    staging_belt: Local<Option<wgpu::util::StagingBelt>>,
    window_specs: Res<WindowSpecs>,
) {
    let staging_belt = staging_belt.get_or_insert(wgpu::util::StagingBelt::new(1024));
    staging_belt.recall();

    if let (Some(glyph_brush), Some(fonts), Some(device), Some(render_context)) = (
        glyph_brush.0.as_mut(),
        fonts,
        device,
        render_context.as_mut(),
    ) {
        for (text, transform) in texts.iter() {
            glyph_brush.queue(Section {
                screen_position: (transform.position.x, transform.position.y),
                bounds: text.bounds.into(),
                text: text
                    .sections
                    .iter()
                    .map(|s| {
                        wgpu_glyph::Text::new(&s.value)
                            .with_color(s.style.color.to_array())
                            .with_scale(s.style.font_size)
                            .with_font_id(fonts.get(&s.style.font).unwrap().font_id.unwrap())
                    })
                    .collect(),
                ..Default::default()
            })
        }

        glyph_brush
            .draw_queued(
                &device,
                staging_belt,
                &mut render_context.command_encoder,
                &render_context.texture_view,
                window_specs.size.x,
                window_specs.size.y,
            )
            .expect("Draw queued");

        staging_belt.finish();
    }
}

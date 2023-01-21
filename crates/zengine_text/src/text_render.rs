use std::borrow::Cow;

use ab_glyph::FontArc;
use glam::Vec2;
use glyph_brush::{BrushAction, BrushError, Extra, FontId, Section};
use log::info;
use rustc_hash::FxHashMap;
use wgpu::Device;
use zengine_asset::{Assets, Handle};
use zengine_graphic::CameraBuffer;
use zengine_macro::Resource;

use crate::{
    pipeline::{Instance, Pipeline},
    Font,
};

#[derive(Resource)]
pub struct TextRender {
    font_handle_to_id: FxHashMap<Handle<Font>, FontId>,
    pipeline: Pipeline,
    glyph_brush: glyph_brush::GlyphBrush<Instance, Extra, ab_glyph::FontArc>,
}

impl std::fmt::Debug for TextRender {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //TODO fix this implementation
        Ok(())
    }
}

impl TextRender {
    pub fn new(
        device: &Device,
        render_format: wgpu::TextureFormat,
        camera_buffer: &CameraBuffer,
        fonts: &mut Assets<Font>,
    ) -> Self {
        let mut new_fonts: Vec<FontArc> = Vec::with_capacity(fonts.len());
        let mut i = 0;
        for (_h, f) in fonts.iter_mut() {
            f.font_id = Some(FontId(i));
            new_fonts.push(f.font.clone());
            i += 1;
        }

        let glyph_brush = glyph_brush::GlyphBrushBuilder::using_fonts(new_fonts).build();
        let (cache_width, cache_height) = glyph_brush.texture_dimensions();
        Self {
            font_handle_to_id: FxHashMap::default(),
            pipeline: Pipeline::new(
                device,
                wgpu::FilterMode::Nearest,
                wgpu::MultisampleState::default(),
                render_format,
                cache_width,
                cache_height,
                camera_buffer,
            ),
            glyph_brush,
        }
    }

    #[inline]
    pub fn queue<'a, S>(&mut self, section: S)
    where
        S: Into<Cow<'a, Section<'a>>>,
    {
        self.glyph_brush.queue(section)
    }

    #[inline]
    pub fn draw_queued(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        camera_buffer: &CameraBuffer,
        camera_scale: &Vec2,
    ) -> Result<(), String> {
        self.process_queued(device, staging_belt, encoder, camera_scale);
        self.pipeline.draw(encoder, target, None, camera_buffer);

        Ok(())
    }

    fn process_queued(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        scale: &Vec2,
    ) {
        let pipeline = &mut self.pipeline;

        let mut brush_action;

        loop {
            brush_action = self.glyph_brush.process_queued(
                |rect, tex_data| {
                    info!("rect {rect:?} ");
                    let offset = [rect.min[0] as u16, rect.min[1] as u16];
                    let size = [rect.width() as u16, rect.height() as u16];

                    pipeline.update_cache(device, staging_belt, encoder, offset, size, tex_data);
                },
                |vertex| Instance::from_vertex(vertex, scale),
            );

            match brush_action {
                Ok(_) => break,
                Err(BrushError::TextureTooSmall { suggested }) => {
                    // TODO: Obtain max texture dimensions using `wgpu`
                    // This is currently not possible I think. Ask!
                    let max_image_dimension = 2048;

                    let (new_width, new_height) = if (suggested.0 > max_image_dimension
                        || suggested.1 > max_image_dimension)
                        && (self.glyph_brush.texture_dimensions().0 < max_image_dimension
                            || self.glyph_brush.texture_dimensions().1 < max_image_dimension)
                    {
                        (max_image_dimension, max_image_dimension)
                    } else {
                        suggested
                    };

                    log::warn!(
                        "Increasing glyph texture size {old:?} -> {new:?}. \
                             Consider building with `.initial_cache_size({new:?})` to avoid \
                             resizing",
                        old = self.glyph_brush.texture_dimensions(),
                        new = (new_width, new_height),
                    );

                    pipeline.increase_cache_size(device, new_width, new_height);
                    self.glyph_brush.resize_texture(new_width, new_height);
                }
            }
        }

        match brush_action.unwrap() {
            BrushAction::Draw(verts) => {
                self.pipeline.upload(device, staging_belt, encoder, &verts);
            }
            BrushAction::ReDraw => {}
        };
    }
}

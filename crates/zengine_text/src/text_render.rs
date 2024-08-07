use etagere::{size2, Allocation};
use fontdue::{
    layout::{GlyphRasterConfig, Layout},
    Font,
};
use glam::{Vec2, Vec4};
use std::{borrow::Borrow, collections::HashSet, slice};
use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, Device, Extent3d, ImageCopyTexture, ImageDataLayout,
    IndexFormat, Origin3d, Queue, RenderPass, TextureAspect, COPY_BUFFER_ALIGNMENT,
};
use zengine_graphic::{CameraBuffer, Vertex};
use zengine_macro::Resource;

use crate::{
    error::{PrepareError, RenderError},
    GlyphDetails, GlyphUserData, GpuCache, TextAtlas,
};

/// A text renderer that uses cached glyphs to render text into an existing render pass.
#[derive(Resource, Debug)]
pub(crate) struct TextRenderer {
    vertex_buffer: Buffer,
    vertex_buffer_size: u64,
    index_buffer: Buffer,
    index_buffer_size: u64,
    vertices_to_render: u32,
    glyphs_in_use: HashSet<GlyphRasterConfig>,
}

impl TextRenderer {
    /// Creates a new `TextRenderer`.
    pub fn new(device: &Device, _queue: &Queue) -> Self {
        let vertex_buffer_size = next_copy_buffer_size(4096);
        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("glyphon vertices"),
            size: vertex_buffer_size,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer_size = next_copy_buffer_size(4096);
        let index_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("glyphon indices"),
            size: index_buffer_size,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            vertex_buffer,
            vertex_buffer_size,
            index_buffer,
            index_buffer_size,
            vertices_to_render: 0,
            glyphs_in_use: HashSet::new(),
        }
    }

    /// Prepares all of the provided layouts for rendering.
    pub(crate) fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        atlas: &mut TextAtlas,
        fonts: &[&Font],
        layouts: &[impl Borrow<Layout<GlyphUserData>>],
        scale: &Vec2,
    ) -> Result<(), PrepareError> {
        struct UploadBounds {
            x_min: usize,
            x_max: usize,
            y_min: usize,
            y_max: usize,
        }
        let mut upload_bounds = None::<UploadBounds>;

        self.glyphs_in_use.clear();

        for layout in layouts.iter() {
            for glyph in layout.borrow().glyphs() {
                self.glyphs_in_use.insert(glyph.key);

                let already_on_gpu = atlas.glyph_cache.contains_key(&glyph.key);

                if already_on_gpu {
                    continue;
                }

                let font = &fonts[glyph.font_index];
                let (metrics, bitmap) = font.rasterize_config(glyph.key);

                let (gpu_cache, atlas_id) = if glyph.char_data.rasterize() {
                    // Find a position in the packer
                    let allocation = match try_allocate(atlas, metrics.width, metrics.height) {
                        Some(a) => a,
                        None => return Err(PrepareError::AtlasFull),
                    };
                    let atlas_min = allocation.rectangle.min;
                    let atlas_max = allocation.rectangle.max;

                    for row in 0..metrics.height {
                        let y_offset = atlas_min.y as usize;
                        let x_offset =
                            (y_offset + row) * atlas.width as usize + atlas_min.x as usize;
                        let bitmap_row = &bitmap[row * metrics.width..(row + 1) * metrics.width];
                        atlas.texture_pending[x_offset..x_offset + metrics.width]
                            .copy_from_slice(bitmap_row);
                    }

                    match upload_bounds.as_mut() {
                        Some(ub) => {
                            ub.x_min = ub.x_min.min(atlas_min.x as usize);
                            ub.x_max = ub.x_max.max(atlas_max.x as usize);
                            ub.y_min = ub.y_min.min(atlas_min.y as usize);
                            ub.y_max = ub.y_max.max(atlas_max.y as usize);
                        }
                        None => {
                            upload_bounds = Some(UploadBounds {
                                x_min: atlas_min.x as usize,
                                x_max: atlas_max.x as usize,
                                y_min: atlas_min.y as usize,
                                y_max: atlas_max.y as usize,
                            });
                        }
                    }

                    (
                        GpuCache::InAtlas {
                            x: atlas_min.x as f32,
                            y: atlas_min.y as f32,
                        },
                        Some(allocation.id),
                    )
                } else {
                    (GpuCache::SkipRasterization, None)
                };

                if !atlas.glyph_cache.contains_key(&glyph.key) {
                    atlas.glyph_cache.insert(
                        glyph.key,
                        GlyphDetails {
                            width: metrics.width as f32,
                            height: metrics.height as f32,
                            gpu_cache,
                            atlas_id,
                        },
                    );
                }
            }
        }

        if let Some(ub) = upload_bounds {
            queue.write_texture(
                ImageCopyTexture {
                    texture: &atlas.texture,
                    mip_level: 0,
                    origin: Origin3d {
                        x: ub.x_min as u32,
                        y: ub.y_min as u32,
                        z: 0,
                    },
                    aspect: TextureAspect::All,
                },
                &atlas.texture_pending[ub.y_min * atlas.width as usize + ub.x_min..],
                ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(atlas.width),
                    rows_per_image: Some(atlas.height),
                },
                Extent3d {
                    width: (ub.x_max - ub.x_min) as u32,
                    height: (ub.y_max - ub.y_min) as u32,
                    depth_or_array_layers: 1,
                },
            );
        }

        let mut glyph_vertices = Vec::new();
        let mut glyph_indices = Vec::new();
        let mut glyphs_added = 0;

        for layout in layouts.iter() {
            let layout: &Layout<GlyphUserData> = layout.borrow();

            for glyph in layout.glyphs() {
                let mut x = glyph.x;
                let mut y = glyph.y;

                let details = atlas.glyph_cache.get(&glyph.key).unwrap();
                let (atlas_x, atlas_y) = match details.gpu_cache {
                    GpuCache::InAtlas { x, y } => (x, y),
                    GpuCache::SkipRasterization => continue,
                };

                let width = details.width;
                let height = details.height;

                let color = glyph.user_data.color;
                let transform = glyph.user_data.transform_matrix;

                x /= scale.x;
                y /= scale.y;
                let s_width = width / scale.x;
                let s_height = height / scale.y;

                glyph_vertices.extend([
                    Vertex {
                        position: transform.mul_vec4(Vec4::new(x, y, 1., 1.)).to_array(),
                        tex_coords: [atlas_x, atlas_y],
                        color: color.to_array(),
                    },
                    Vertex {
                        position: transform
                            .mul_vec4(Vec4::new(x + s_width, y, 1., 1.))
                            .to_array(),
                        tex_coords: [atlas_x + width, atlas_y],
                        color: color.to_array(),
                    },
                    Vertex {
                        position: transform
                            .mul_vec4(Vec4::new(x + s_width, y + s_height, 1., 1.))
                            .to_array(),
                        tex_coords: [atlas_x + width, atlas_y + height],
                        color: color.to_array(),
                    },
                    Vertex {
                        position: transform
                            .mul_vec4(Vec4::new(x, y + s_height, 1., 1.))
                            .to_array(),
                        tex_coords: [atlas_x, atlas_y + height],
                        color: color.to_array(),
                    },
                ]);

                let start = 4 * glyphs_added as u32;
                glyph_indices.extend([start, start + 1, start + 2, start, start + 2, start + 3]);

                glyphs_added += 1;
            }
        }

        const VERTICES_PER_GLYPH: u32 = 6;
        self.vertices_to_render = glyphs_added as u32 * VERTICES_PER_GLYPH;

        let will_render = glyphs_added > 0;
        if !will_render {
            return Ok(());
        }

        let vertices = glyph_vertices.as_slice();
        let vertices_raw = unsafe {
            slice::from_raw_parts(
                vertices as *const _ as *const u8,
                std::mem::size_of_val(vertices),
            )
        };

        if self.vertex_buffer_size >= vertices_raw.len() as u64 {
            queue.write_buffer(&self.vertex_buffer, 0, vertices_raw);
        } else {
            self.vertex_buffer.destroy();

            let (buffer, buffer_size) = create_oversized_buffer(
                device,
                Some("glyphon vertices"),
                vertices_raw,
                BufferUsages::VERTEX | BufferUsages::COPY_DST,
            );

            self.vertex_buffer = buffer;
            self.vertex_buffer_size = buffer_size;
        }

        let indices = glyph_indices.as_slice();
        let indices_raw = unsafe {
            slice::from_raw_parts(
                indices as *const _ as *const u8,
                std::mem::size_of_val(indices),
            )
        };

        if self.index_buffer_size >= indices_raw.len() as u64 {
            queue.write_buffer(&self.index_buffer, 0, indices_raw);
        } else {
            self.index_buffer.destroy();

            let (buffer, buffer_size) = create_oversized_buffer(
                device,
                Some("glyphon indices"),
                indices_raw,
                BufferUsages::INDEX | BufferUsages::COPY_DST,
            );

            self.index_buffer = buffer;
            self.index_buffer_size = buffer_size;
        }

        Ok(())
    }

    /// Renders all layouts that were previously provided to `prepare`.
    pub fn render<'pass>(
        &'pass mut self,
        atlas: &'pass TextAtlas,
        pass: &mut RenderPass<'pass>,
        camera_buffer: &'pass CameraBuffer,
    ) -> Result<(), RenderError> {
        if self.vertices_to_render == 0 {
            return Ok(());
        }

        {
            // Validate that glyphs haven't been evicted from cache since `prepare`
            for glyph in self.glyphs_in_use.iter() {
                if !atlas.glyph_cache.contains_key(glyph) {
                    return Err(RenderError::RemovedFromAtlas);
                }
            }
        }

        pass.set_pipeline(&atlas.pipeline);
        pass.set_bind_group(0, &atlas.bind_group, &[]);
        pass.set_bind_group(1, &camera_buffer.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        pass.draw_indexed(0..self.vertices_to_render, 0, 0..1);

        Ok(())
    }
}

fn try_allocate(atlas: &mut TextAtlas, width: usize, height: usize) -> Option<Allocation> {
    let size = size2(width as i32, height as i32);

    loop {
        let allocation = atlas.packer.allocate(size);
        if allocation.is_some() {
            return allocation;
        }

        // Try to free least recently used allocation
        let (key, value) = atlas.glyph_cache.pop()?;
        atlas
            .packer
            .deallocate(value.atlas_id.expect("cache corrupt"));
        atlas.glyph_cache.remove(&key);
    }
}

fn next_copy_buffer_size(size: u64) -> u64 {
    let align_mask = COPY_BUFFER_ALIGNMENT - 1;
    ((size.next_power_of_two() + align_mask) & !align_mask).max(COPY_BUFFER_ALIGNMENT)
}

fn create_oversized_buffer(
    device: &Device,
    label: Option<&str>,
    contents: &[u8],
    usage: BufferUsages,
) -> (Buffer, u64) {
    let size = next_copy_buffer_size(contents.len() as u64);
    let buffer = device.create_buffer(&BufferDescriptor {
        label,
        size,
        usage,
        mapped_at_creation: true,
    });
    buffer.slice(..).get_mapped_range_mut()[..contents.len()].copy_from_slice(contents);
    buffer.unmap();
    (buffer, size)
}

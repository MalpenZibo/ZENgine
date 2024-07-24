use etagere::{size2, BucketedAtlasAllocator};
use fontdue::layout::GlyphRasterConfig;
use std::{borrow::Cow, fmt::Debug, sync::Arc};
use wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayoutEntry, BindingResource, BindingType, BlendState,
    ColorTargetState, ColorWrites, Device, Extent3d, FilterMode, FragmentState, MultisampleState,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPipeline, RenderPipelineDescriptor,
    SamplerBindingType, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages,
    Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
    TextureViewDescriptor, TextureViewDimension, VertexState,
};
use zengine_graphic::{CameraBuffer, Vertex};
use zengine_macro::Resource;

use crate::{recently_used::RecentlyUsedMap, GlyphDetails};

/// An atlas containing a cache of rasterized glyphs that can be rendered.
#[derive(Resource)]
pub(crate) struct TextAtlas {
    pub texture_pending: Vec<u8>,
    pub texture: Texture,
    pub packer: BucketedAtlasAllocator,
    pub width: u32,
    pub height: u32,
    pub glyph_cache: RecentlyUsedMap<GlyphRasterConfig, GlyphDetails>,
    pub pipeline: Arc<RenderPipeline>,
    pub bind_group: Arc<BindGroup>,
}

impl Debug for TextAtlas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TextAtlas")
    }
}

impl TextAtlas {
    /// Creates a new `TextAtlas`.
    pub fn new(
        device: &Device,
        _queue: &Queue,
        format: TextureFormat,
        camera_buffer: &CameraBuffer,
    ) -> Self {
        let max_texture_dimension_2d = device.limits().max_texture_dimension_2d;
        let width = max_texture_dimension_2d;
        let height = max_texture_dimension_2d;

        let packer = BucketedAtlasAllocator::new(size2(width as i32, height as i32));
        // Create a texture to use for our atlas
        let texture_pending = vec![0; (width * height) as usize];
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("glyphon atlas"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("glyphon sampler"),
            min_filter: FilterMode::Nearest,
            mag_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0f32,
            lod_max_clamp: 0f32,
            ..Default::default()
        });

        let glyph_cache = RecentlyUsedMap::new();

        // Create a render pipeline to use for rendering later
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("glyphon shader"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let vertex_buffers = [Vertex::desc()];

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("glyphon bind group layout"),
        });

        let bind_group = Arc::new(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("glyphon bind group"),
        }));

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout, &camera_buffer.bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = Arc::new(device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("glyphon pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &vertex_buffers,
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: ColorWrites::default(),
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None
        }));

        Self {
            texture_pending,
            texture,
            packer,
            width,
            height,
            glyph_cache,
            pipeline,
            bind_group,
        }
    }
}

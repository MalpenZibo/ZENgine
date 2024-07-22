use crate::{
    vertex::Vertex, CameraBuffer, Color, Device, Image, Queue, RenderContextInstance, Surface,
    Texture, TextureAtlas, TextureBindGroupLayout,
};
use glam::{Mat4, Vec2, Vec3, Vec4};
use rustc_hash::FxHashMap;
use std::ops::{Deref, DerefMut};
use wgpu::util::DeviceExt;
use zengine_asset::{Assets, Handle};
use zengine_core::Transform;
use zengine_ecs::{
    query::{Query, QueryIter},
    system::{Commands, Local, Res, ResMut},
};
use zengine_macro::{Component, Resource};

const VERTICES: &[Vec4; 4] = &[
    Vec4::new(-0.5, 0.5, 0.0, 1.0),
    Vec4::new(-0.5, -0.5, 0.0, 1.0),
    Vec4::new(0.5, -0.5, 0.0, 1.0),
    Vec4::new(0.5, 0.5, 0.0, 1.0),
];
const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

struct SpriteTextureInfo {
    size: Vec2,
    ratio: f32,
    min: Vec2,
    max: Vec2,
}

/// A sprite texture
///
/// It could be a simple texture
/// or it could be an Atlas texture
#[derive(Debug)]
pub enum SpriteTexture {
    /// Handle to the [Texture] asset
    Simple(Handle<Texture>),
    Atlas {
        /// handle to the [TextureAtlas] asset
        texture_handle: Handle<TextureAtlas>,
        /// optional handle to the [Image] asset contained in the Atlas
        target_image: Option<Handle<Image>>,
    },
}

impl SpriteTexture {
    fn is_ready(&self, textures: &Assets<Texture>, textures_atlas: &Assets<TextureAtlas>) -> bool {
        match self {
            Self::Simple(handle) => textures
                .get(handle)
                .and_then(|t| t.gpu_image.as_ref())
                .is_some(),
            Self::Atlas { texture_handle, .. } => textures_atlas
                .get(texture_handle)
                .and_then(|t| t.texture.as_ref())
                .is_some(),
        }
    }

    fn get_handle(&self, textures_atlas: &Assets<TextureAtlas>) -> Handle<Texture> {
        match self {
            Self::Simple(handle) => handle.clone_as_weak(),
            Self::Atlas { texture_handle, .. } => textures_atlas
                .get(texture_handle)
                .and_then(|t| t.texture.as_ref())
                .unwrap()
                .clone_as_weak(),
        }
    }

    fn get_info(
        &self,
        texture: &Texture,
        textures_atlas: &Assets<TextureAtlas>,
    ) -> SpriteTextureInfo {
        match self {
            Self::Simple(_) => SpriteTextureInfo {
                size: texture.size,
                ratio: texture.ratio,
                min: Vec2::ZERO,
                max: Vec2::ONE,
            },
            Self::Atlas {
                texture_handle,
                target_image,
            } => textures_atlas
                .get(texture_handle)
                .and_then(|t| {
                    target_image
                        .as_ref()
                        .map(|target_image| t.get_rect(target_image))
                })
                .map(|rect| SpriteTextureInfo {
                    size: rect.size,
                    ratio: rect.ratio,
                    min: rect.relative_min,
                    max: rect.relative_max,
                })
                .unwrap_or_else(|| SpriteTextureInfo {
                    size: Vec2::ZERO,
                    ratio: 1.,
                    min: Vec2::ZERO,
                    max: Vec2::ONE,
                }),
        }
    }
}

/// Rappresent a Sprite size
#[derive(Debug)]
pub enum SpriteSize {
    /// Use the image size as sprite size
    None,
    /// Set the sprite width, the height is automatically calculated based on texture ratio
    Width(f32),
    /// Set the sprite height, the width is automatically calculated based on texture ratio
    Height(f32),
    /// Set the sprite size specifying both the width and height
    Size(Vec2),
}

/// [Component](zengine_ecs::Component) that rappresent a Sprite
#[derive(Component, Debug)]
pub struct Sprite {
    /// The sprite size
    pub size: SpriteSize,
    /// origin of the sprite, indicate the center of the sprite
    pub origin: glam::Vec3,
    /// color to apply to the sprite texture
    pub color: Color,
    /// texture to use with this sprite
    pub texture: SpriteTexture,
}

#[doc(hidden)]
#[derive(Resource, Default, Debug)]
pub struct SpriteBuffer {
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    size: usize,
}

type BatchLayerData<'a> = FxHashMap<Handle<Texture>, Vec<(&'a Sprite, &'a Transform)>>;

struct BatchLayer<'a> {
    pub z: f32,
    pub data: BatchLayerData<'a>,
}

#[doc(hidden)]
#[derive(Resource, Debug)]
pub struct RenderPipeline(wgpu::RenderPipeline);

impl Deref for RenderPipeline {
    type Target = wgpu::RenderPipeline;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn calculate_vertices(
    size: &SpriteSize,
    origin: Vec3,
    info: SpriteTextureInfo,
    color: &Color,
    transform: Mat4,
) -> [Vertex; 4] {
    let (width, height) = {
        match size {
            SpriteSize::Width(w) => (*w, w / info.ratio),
            SpriteSize::Height(h) => (h * info.ratio, *h),
            SpriteSize::Size(size) => (size.x, size.y),
            SpriteSize::None => (info.size.x, info.size.y),
        }
    };

    let min_u = info.min.x;
    let max_u = info.max.x;

    let min_v = info.min.y;
    let max_v = info.max.y;

    let origin_matrix = Mat4::from_translation(origin / 2.).inverse();
    let scale_matrix = Mat4::from_scale(Vec3::new(width, height, 1.0));

    [
        Vertex {
            position: transform
                .mul_vec4(scale_matrix.mul_vec4(origin_matrix.mul_vec4(VERTICES[0])))
                .to_array(),
            tex_coords: [min_u, min_v],
            color: color.to_array(),
        },
        Vertex {
            position: transform
                .mul_vec4(scale_matrix.mul_vec4(origin_matrix.mul_vec4(VERTICES[1])))
                .to_array(),
            tex_coords: [min_u, max_v],
            color: color.to_array(),
        },
        Vertex {
            position: transform
                .mul_vec4(scale_matrix.mul_vec4(origin_matrix.mul_vec4(VERTICES[2])))
                .to_array(),
            tex_coords: [max_u, max_v],
            color: color.to_array(),
        },
        Vertex {
            position: transform
                .mul_vec4(scale_matrix.mul_vec4(origin_matrix.mul_vec4(VERTICES[3])))
                .to_array(),
            tex_coords: [max_u, min_v],
            color: color.to_array(),
        },
    ]
}

fn generate_vertex_and_indexes_buffer(
    device: &wgpu::Device,
    vertex: &[Vertex],
) -> (wgpu::Buffer, wgpu::Buffer) {
    let mut indices = Vec::default();
    for index in 0..vertex.iter().len() / 4 {
        indices.extend(INDICES.iter().map(|i| i + (4 * index as u16)))
    }

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(vertex),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
    });

    (vertex_buffer, index_buffer)
}

pub(crate) fn setup_sprite_render(
    surface: Res<Surface>,
    texture_bind_group_layout: Option<Res<TextureBindGroupLayout>>,
    device: Option<Res<Device>>,
    camera_buffer: Option<Res<CameraBuffer>>,
    mut commands: Commands,
) {
    let config = surface.get_config().unwrap();
    let device = device.unwrap();
    let camera_buffer = camera_buffer.unwrap();

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
            &texture_bind_group_layout.unwrap(),
            &camera_buffer.bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    commands.create_resource(RenderPipeline(render_pipeline));
}

#[derive(Default)]
struct Batches<'a>(Vec<BatchLayer<'a>>);
impl<'a> Batches<'a> {
    fn to_vertex(
        &self,
        textures: &Assets<Texture>,
        textures_atlas: &Assets<TextureAtlas>,
    ) -> Vec<Vertex> {
        self.0
            .iter()
            .rev()
            .flat_map(|b| {
                b.data.iter().flat_map(|(t, v)| {
                    let texture = textures.get(t).unwrap();
                    v.iter().flat_map(|(s, t)| {
                        let info = s.texture.get_info(texture, textures_atlas);

                        calculate_vertices(
                            &s.size,
                            s.origin,
                            info,
                            &s.color,
                            t.get_transformation_matrix(),
                        )
                    })
                })
            })
            .collect()
    }
}

impl<'a> Deref for Batches<'a> {
    type Target = Vec<BatchLayer<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for Batches<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn sprite_render(
    queue: Option<Res<Queue>>,
    device: Option<Res<Device>>,
    mut render_context: ResMut<RenderContextInstance>,
    render_pipeline: Option<Res<RenderPipeline>>,
    textures: Option<Res<Assets<Texture>>>,
    textures_atlas: Option<Res<Assets<TextureAtlas>>>,
    camera_buffer: Option<Res<CameraBuffer>>,
    sprite_query: Query<(&Sprite, &Transform)>,
    sprite_buffer: Local<SpriteBuffer>,
) {
    if let (
        Some(textures),
        Some(textures_atlas),
        Some(device),
        Some(queue),
        Some(camera_buffer),
        Some(render_pipeline),
    ) = (
        textures,
        textures_atlas,
        device,
        queue,
        camera_buffer,
        render_pipeline,
    ) {
        if let Some(render_context) = render_context.as_mut() {
            let mut render_pass =
                render_context
                    .command_encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Sprite Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &render_context.texture_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        ..Default::default()
                    });

            let mut batches = Batches::default();
            for (s, t) in sprite_query.iter() {
                if s.texture.is_ready(&textures, &textures_atlas) {
                    let z = t.position.z;

                    let batch_layer = match batches.binary_search_by(|l| l.z.total_cmp(&z)) {
                        Ok(index) => batches.get_mut(index).unwrap(),
                        Err(index) => {
                            batches.insert(
                                index,
                                BatchLayer {
                                    z,
                                    data: FxHashMap::default(),
                                },
                            );
                            batches.get_mut(index).unwrap()
                        }
                    };

                    match batch_layer
                        .data
                        .get_mut(&s.texture.get_handle(&textures_atlas))
                    {
                        Some(batch) => {
                            batch.push((s, t));
                        }
                        None => {
                            batch_layer
                                .data
                                .insert(s.texture.get_handle(&textures_atlas), vec![(s, t)]);
                        }
                    }
                }
            }

            let num_of_sprite = batches
                .iter()
                .flat_map(|l| l.data.values().map(|v| v.len()))
                .sum();
            if sprite_buffer.size >= num_of_sprite && sprite_buffer.vertex_buffer.is_some() {
                queue.write_buffer(
                    sprite_buffer.vertex_buffer.as_ref().unwrap(),
                    0,
                    bytemuck::cast_slice(&batches.to_vertex(&textures, &textures_atlas)),
                );
                let mut indices = Vec::default();
                for index in 0..num_of_sprite {
                    indices.extend(INDICES.iter().map(|i| i + (4 * index as u16)))
                }

                queue.write_buffer(
                    sprite_buffer.index_buffer.as_ref().unwrap(),
                    0,
                    bytemuck::cast_slice(&indices),
                );
            } else {
                let (v_buffer, i_buffer) = generate_vertex_and_indexes_buffer(
                    &device,
                    &batches.to_vertex(&textures, &textures_atlas),
                );
                if let Some(buffer) = &sprite_buffer.vertex_buffer {
                    buffer.destroy();
                }
                if let Some(buffer) = &sprite_buffer.index_buffer {
                    buffer.destroy();
                }
                sprite_buffer.vertex_buffer = Some(v_buffer);
                sprite_buffer.index_buffer = Some(i_buffer);

                sprite_buffer.size = num_of_sprite;
            }

            render_pass.set_pipeline(&render_pipeline);
            render_pass.set_bind_group(1, &camera_buffer.bind_group, &[]);

            render_pass
                .set_vertex_buffer(0, sprite_buffer.vertex_buffer.as_ref().unwrap().slice(..));
            render_pass.set_index_buffer(
                sprite_buffer.index_buffer.as_ref().unwrap().slice(..),
                wgpu::IndexFormat::Uint16,
            );

            let mut offset: u32 = 0;
            for b in batches.iter().rev() {
                for (k, v) in b.data.iter() {
                    let texture = textures
                        .get(k)
                        .and_then(|t1| t1.gpu_image.as_ref())
                        .unwrap();

                    let elements = v.len() as u32;
                    let i_offset = offset * 6;
                    let max_i = elements * 6 + i_offset;

                    render_pass.set_bind_group(0, &texture.diffuse_bind_group, &[]);
                    render_pass.draw_indexed(i_offset..max_i, 0, 0..1);

                    offset += elements;
                }
            }
        }
    }
}

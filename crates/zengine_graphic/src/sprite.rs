use std::ops::Deref;

use glam::{Mat4, Vec2, Vec3, Vec4};
use rustc_hash::FxHashMap;
use wgpu::util::DeviceExt;
use zengine_core::Transform;
use zengine_ecs::system::{Commands, Local, OptionalRes, Query, QueryIter, ResMut, UnsendableRes};
use zengine_macro::Resource;

use crate::{
    vertex::Vertex, CameraBuffer, Color, Device, Queue, RenderContextInstance, Sprite,
    SpriteHandle, SpriteType, SurfaceData, TextureBindGroupLayout, TextureHandleState,
    TextureManager,
};

pub const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

#[derive(Resource, Default, Debug)]
pub struct SpriteBuffer {
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    size: usize,
}

type BatchLayerData<'a, ST> =
    FxHashMap<usize, Vec<(&'a Sprite<ST>, &'a Transform, &'a SpriteHandle)>>;

pub struct BatchLayer<'a, ST: SpriteType> {
    pub z: f32,
    pub data: BatchLayerData<'a, ST>,
}

#[derive(Resource, Debug)]
pub struct RenderPipeline(wgpu::RenderPipeline);

impl Deref for RenderPipeline {
    type Target = wgpu::RenderPipeline;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn calculate_vertices(
    width: f32,
    height: f32,
    origin: Vec3,
    relative_min: Vec2,
    relative_max: Vec2,
    color: &Color,
    transform: Mat4,
) -> [Vertex; 4] {
    let min_x = -(width * origin.x);
    let max_x = width * (1.0 - origin.x);

    let min_y = -(height * origin.y);
    let max_y = height * (1.0 - origin.y);

    let min_u = relative_min.x;
    let max_u = relative_max.x;

    let min_v = relative_min.y;
    let max_v = relative_max.y;

    [
        Vertex {
            position: transform
                .mul_vec4(Vec4::new(min_x, max_y, 0.0, 1.0))
                .to_array(),
            tex_coords: [min_u, min_v],
            color: color.to_array(),
        },
        Vertex {
            position: transform
                .mul_vec4(Vec4::new(min_x, min_y, 0.0, 1.0))
                .to_array(),
            tex_coords: [min_u, max_v],
            color: color.to_array(),
        },
        Vertex {
            position: transform
                .mul_vec4(Vec4::new(max_x, min_y, 0.0, 1.0))
                .to_array(),
            tex_coords: [max_u, max_v],
            color: color.to_array(),
        },
        Vertex {
            position: transform
                .mul_vec4(Vec4::new(max_x, max_y, 0.0, 1.0))
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

pub fn setup_sprite_render(
    surface: OptionalRes<SurfaceData>,
    texture_bind_group_layout: OptionalRes<TextureBindGroupLayout>,
    device: OptionalRes<Device>,
    camera_buffer: OptionalRes<CameraBuffer>,
    mut commands: Commands,
) {
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
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: surface.unwrap().surface_config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
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

#[allow(clippy::too_many_arguments)]
pub fn sprite_render<ST: SpriteType>(
    queue: OptionalRes<Queue>,
    device: OptionalRes<Device>,
    mut render_context: ResMut<RenderContextInstance>,
    render_pipeline: OptionalRes<RenderPipeline>,
    texture_manager: UnsendableRes<TextureManager<ST>>,
    camera_buffer: OptionalRes<CameraBuffer>,
    sprite_query: Query<(&Sprite<ST>, &Transform)>,
    sprite_buffer: Local<SpriteBuffer>,
) {
    if let (Some(device), Some(queue), Some(camera_buffer), Some(render_pipeline)) =
        (device, queue, camera_buffer, render_pipeline)
    {
        let render_context = render_context.as_mut().unwrap();
        {
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
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });

            let mut batches: Vec<BatchLayer<ST>> = Vec::default();
            for (s, t) in sprite_query.iter() {
                if let Some(sprite_handle) = texture_manager.get_sprite_handle(&s.sprite_type) {
                    if texture_manager
                        .textures
                        .get(sprite_handle.texture_handle_index)
                        .unwrap()
                        .state
                        == TextureHandleState::Uploaded
                    {
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
                            .get_mut(&sprite_handle.texture_handle_index)
                        {
                            Some(batch) => {
                                batch.push((s, t, sprite_handle));
                            }
                            None => {
                                batch_layer.data.insert(
                                    sprite_handle.texture_handle_index,
                                    vec![(s, t, sprite_handle)],
                                );
                            }
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
                    bytemuck::cast_slice(
                        &batches
                            .iter()
                            .rev()
                            .flat_map(|b| {
                                b.data.values().flat_map(|v| {
                                    v.iter().flat_map(|(s, t, sprite_handle)| {
                                        calculate_vertices(
                                            s.width,
                                            s.height,
                                            s.origin,
                                            sprite_handle.relative_min,
                                            sprite_handle.relative_max,
                                            &s.color,
                                            t.get_transformation_matrix(),
                                        )
                                    })
                                })
                            })
                            .collect::<Vec<Vertex>>(),
                    ),
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
                    &batches
                        .iter()
                        .rev()
                        .flat_map(|b| {
                            b.data.values().flat_map(|v| {
                                v.iter().flat_map(|(s, t, sprite_handle)| {
                                    calculate_vertices(
                                        s.width,
                                        s.height,
                                        s.origin,
                                        sprite_handle.relative_min,
                                        sprite_handle.relative_max,
                                        &s.color,
                                        t.get_transformation_matrix(),
                                    )
                                })
                            })
                        })
                        .collect::<Vec<Vertex>>(),
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
                    let texture = texture_manager
                        .textures
                        .get(*k)
                        .and_then(|t1| t1.texture.as_ref())
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

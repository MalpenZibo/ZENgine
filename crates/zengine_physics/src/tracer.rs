use std::{collections::HashSet, f32::consts::PI, ops::Deref};

use glam::{Mat4, Vec3, Vec4};
use wgpu::util::DeviceExt;
use zengine_core::Transform;
use zengine_ecs::{
    query::{Query, QueryIter},
    system::{Commands, EventStream, Local, Res, ResMut},
    Entity,
};
use zengine_graphic::{CameraBuffer, Color, Device, Queue, RenderContextInstance, Surface};
use zengine_macro::Resource;

use crate::{Collision, Shape2D, ShapeType};

const VERTICES: &[Vec4; 4] = &[
    Vec4::new(-0.5, 0.5, 0.0, 1.0),
    Vec4::new(-0.5, -0.5, 0.0, 1.0),
    Vec4::new(0.5, -0.5, 0.0, 1.0),
    Vec4::new(0.5, 0.5, 0.0, 1.0),
];
const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

const SEGMENTS: usize = 36;

#[doc(hidden)]
#[derive(Resource, Debug)]
pub struct TracerPipeline(wgpu::RenderPipeline);

impl Deref for TracerPipeline {
    type Target = wgpu::RenderPipeline;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 4],
    pub color: [f32; 4],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

fn calculate_rect_vertices(
    width: f32,
    height: f32,
    origin: Vec3,
    color: &Color,
    transform: Mat4,
) -> Vec<Vertex> {
    let origin_matrix = Mat4::from_translation(origin / 2.).inverse();
    let scale_matrix = Mat4::from_scale(Vec3::new(width, height, 1.0));

    vec![
        Vertex {
            position: transform
                .mul_vec4(scale_matrix.mul_vec4(origin_matrix.mul_vec4(VERTICES[0])))
                .to_array(),
            color: color.to_array(),
        },
        Vertex {
            position: transform
                .mul_vec4(scale_matrix.mul_vec4(origin_matrix.mul_vec4(VERTICES[1])))
                .to_array(),
            color: color.to_array(),
        },
        Vertex {
            position: transform
                .mul_vec4(scale_matrix.mul_vec4(origin_matrix.mul_vec4(VERTICES[2])))
                .to_array(),
            color: color.to_array(),
        },
        Vertex {
            position: transform
                .mul_vec4(scale_matrix.mul_vec4(origin_matrix.mul_vec4(VERTICES[3])))
                .to_array(),
            color: color.to_array(),
        },
    ]
}

fn calculate_circle_vertices(
    radius: f32,
    origin: Vec3,
    color: &Color,
    transform: Mat4,
) -> Vec<Vertex> {
    let origin_matrix = Mat4::from_translation(origin).inverse();
    let scale_matrix = Mat4::from_scale(Vec3::new(radius, radius, 1.0));

    let mut vertices = Vec::with_capacity(SEGMENTS + 1);
    let angle_increment = 2.0 * PI / SEGMENTS as f32;

    let center = Vec3::ZERO.extend(1.0);
    //
    // Center vertex at the origin
    vertices.push(Vertex {
        position: transform
            .mul_vec4(scale_matrix.mul_vec4(origin_matrix.mul_vec4(center)))
            .to_array(),
        color: color.to_array(),
    });

    for i in 0..SEGMENTS {
        let theta = i as f32 * angle_increment;

        let x = 1. * theta.cos();
        let y = 1. * theta.sin();
        let z = 0.0;

        vertices.push(Vertex {
            position: transform
                .mul_vec4(scale_matrix.mul_vec4(origin_matrix.mul_vec4(Vec4::new(x, y, z, 1.0))))
                .to_array(),
            color: color.to_array(),
        });
    }

    vertices
}

fn calculate_circle_indices() -> Vec<u16> {
    let mut indices: Vec<u16> = Vec::with_capacity(SEGMENTS * 3);

    for i in 1..=SEGMENTS {
        let next_index = if i == SEGMENTS { 1 } else { i + 1 };
        indices.push(0); // Center vertex index
        indices.push(i as u16); // Current vertex index
        indices.push(next_index as u16); // Next vertex index
    }

    indices
}

fn generate_vertex_and_indexes_buffer(
    device: &wgpu::Device,
    indices: &[u16],
    vertex: &[Vertex],
) -> (wgpu::Buffer, wgpu::Buffer) {
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(vertex),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(indices),
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
    });

    (vertex_buffer, index_buffer)
}

pub(crate) fn setup_trace_render(
    surface: Res<Surface>,
    device: Option<Res<Device>>,
    camera_buffer: Option<Res<CameraBuffer>>,
    mut commands: Commands,
) {
    let config = surface.get_config().unwrap();
    let device = device.unwrap();
    let camera_buffer = camera_buffer.unwrap();

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Collision Tracer Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&camera_buffer.bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Collision Tracer Render Pipeline"),
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
        cache: None,
    });

    commands.create_resource(TracerPipeline(render_pipeline));
}

#[doc(hidden)]
#[derive(Resource, Default, Debug)]
pub struct TracerBuffer {
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    v_buff_size: usize,
    i_buff_size: usize,
}

pub fn collision_tracer(
    queue: Option<Res<Queue>>,
    device: Option<Res<Device>>,
    mut render_context: ResMut<RenderContextInstance>,
    tracer_pipeline: Option<Res<TracerPipeline>>,
    camera_buffer: Option<Res<CameraBuffer>>,
    shape_query: Query<(Entity, &Shape2D, &Transform)>,
    tracer_buffer: Local<TracerBuffer>,
    collision_event: EventStream<Collision>,
) {
    let mut collided = HashSet::new();
    for collision in collision_event.read() {
        collided.insert(collision.entity_a);
        collided.insert(collision.entity_b);
    }

    if let (Some(device), Some(queue), Some(camera_buffer), Some(tracer_pipeline)) =
        (device, queue, camera_buffer, tracer_pipeline)
    {
        if let Some(render_context) = render_context.as_mut() {
            let mut render_pass =
                render_context
                    .command_encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Collision tracer Render Pass"),
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

            let num_of_rect_shapes = shape_query
                .iter()
                .filter(|(_, s, _)| matches!(s.shape_type, ShapeType::Rectangle { .. }))
                .count();
            let num_of_circle_shapes = shape_query
                .iter()
                .filter(|(_, s, _)| matches!(s.shape_type, ShapeType::Circle { .. }))
                .count();
            let normal_color = Color::new(0, 255, 0, 100);
            let collided_color = Color::new(255, 0, 0, 100);
            let mut data =
                Vec::with_capacity(num_of_rect_shapes * 4 + num_of_circle_shapes * (SEGMENTS + 1));
            data.extend(
                shape_query
                    .iter()
                    .filter_map(|(e, s, t)| {
                        if let ShapeType::Rectangle { width, height } = &s.shape_type {
                            Some(calculate_rect_vertices(
                                *width,
                                *height,
                                s.origin,
                                if collided.contains(e) {
                                    &collided_color
                                } else {
                                    &normal_color
                                },
                                t.get_transformation_matrix(),
                            ))
                        } else {
                            None
                        }
                    })
                    .flatten(),
            );
            data.extend(
                shape_query
                    .iter()
                    .filter_map(|(e, s, t)| {
                        if let ShapeType::Circle { radius } = &s.shape_type {
                            Some(calculate_circle_vertices(
                                *radius,
                                s.origin,
                                if collided.contains(e) {
                                    &collided_color
                                } else {
                                    &normal_color
                                },
                                t.get_transformation_matrix(),
                            ))
                        } else {
                            None
                        }
                    })
                    .flatten(),
            );

            let mut indices = Vec::with_capacity(
                num_of_rect_shapes * INDICES.len() + num_of_circle_shapes * (SEGMENTS * 3),
            );
            for index in 0..num_of_rect_shapes {
                indices.extend(INDICES.iter().map(|i| i + (4 * index as u16)))
            }
            let circle_indices = calculate_circle_indices();
            let base = (num_of_rect_shapes * 4) as u16;
            for index in 0..num_of_circle_shapes {
                indices.extend(
                    circle_indices
                        .iter()
                        .map(|i| i + base + ((SEGMENTS as u16 + 1) * index as u16)),
                )
            }
            if tracer_buffer.v_buff_size >= data.len()
                && tracer_buffer.i_buff_size >= indices.len()
                && tracer_buffer.vertex_buffer.is_some()
            {
                queue.write_buffer(
                    tracer_buffer.vertex_buffer.as_ref().unwrap(),
                    0,
                    bytemuck::cast_slice(&data),
                );

                queue.write_buffer(
                    tracer_buffer.index_buffer.as_ref().unwrap(),
                    0,
                    bytemuck::cast_slice(&indices),
                );
            } else {
                let (v_buffer, i_buffer) =
                    generate_vertex_and_indexes_buffer(&device, &indices, &data);
                if let Some(buffer) = &tracer_buffer.vertex_buffer {
                    buffer.destroy();
                }
                if let Some(buffer) = &tracer_buffer.index_buffer {
                    buffer.destroy();
                }
                tracer_buffer.vertex_buffer = Some(v_buffer);
                tracer_buffer.index_buffer = Some(i_buffer);

                tracer_buffer.v_buff_size = data.len();
                tracer_buffer.i_buff_size = indices.len();
            }

            render_pass.set_pipeline(&tracer_pipeline);
            render_pass.set_bind_group(0, &camera_buffer.bind_group, &[]);

            render_pass
                .set_vertex_buffer(0, tracer_buffer.vertex_buffer.as_ref().unwrap().slice(..));
            render_pass.set_index_buffer(
                tracer_buffer.index_buffer.as_ref().unwrap().slice(..),
                wgpu::IndexFormat::Uint16,
            );

            render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        }
    }
}

use std::ops::Deref;

use glam::{Mat4, Vec3, Vec4};
use wgpu::util::DeviceExt;
use zengine_core::Transform;
use zengine_ecs::{
    query::{Query, QueryIter},
    system::{Commands, Local, Res, ResMut},
};
use zengine_graphic::{CameraBuffer, Color, Device, Queue, RenderContextInstance, Surface};
use zengine_macro::Resource;

use crate::{Shape2D, ShapeType};

const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

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

fn calculate_vertices(
    size: &ShapeType,
    origin: Vec3,
    color: &Color,
    transform: Mat4,
) -> [Vertex; 4] {
    let (width, height) = {
        match size {
            ShapeType::Rectangle { width, height } => (*width, *height),
            ShapeType::Circle { radius } => (radius * 2.0, radius * 2.0),
        }
    };

    let min_x = -(width * origin.x);
    let max_x = width * (1.0 - origin.x);

    let min_y = -(height * origin.y);
    let max_y = height * (1.0 - origin.y);

    [
        Vertex {
            position: transform
                .mul_vec4(Vec4::new(min_x, max_y, 0.0, 1.0))
                .to_array(),
            color: color.to_array(),
        },
        Vertex {
            position: transform
                .mul_vec4(Vec4::new(min_x, min_y, 0.0, 1.0))
                .to_array(),
            color: color.to_array(),
        },
        Vertex {
            position: transform
                .mul_vec4(Vec4::new(max_x, min_y, 0.0, 1.0))
                .to_array(),
            color: color.to_array(),
        },
        Vertex {
            position: transform
                .mul_vec4(Vec4::new(max_x, max_y, 0.0, 1.0))
                .to_array(),
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
    });

    commands.create_resource(TracerPipeline(render_pipeline));
}

#[doc(hidden)]
#[derive(Resource, Default, Debug)]
pub struct TracerBuffer {
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    size: usize,
}

pub(crate) fn collision_tracer(
    queue: Option<Res<Queue>>,
    device: Option<Res<Device>>,
    mut render_context: ResMut<RenderContextInstance>,
    tracer_pipeline: Option<Res<TracerPipeline>>,
    camera_buffer: Option<Res<CameraBuffer>>,
    shape_query: Query<(&Shape2D, &Transform)>,
    tracer_buffer: Local<TracerBuffer>,
) {
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

            let num_of_shapes = shape_query.iter().count();
            let color = Color::new(0, 255, 0, 100);
            let data = shape_query
                .iter()
                .flat_map(|(s, t)| {
                    calculate_vertices(
                        &s.shape_type,
                        s.origin,
                        &color,
                        t.get_transformation_matrix(),
                    )
                })
                .collect::<Vec<_>>();

            if tracer_buffer.size >= num_of_shapes && tracer_buffer.vertex_buffer.is_some() {
                queue.write_buffer(
                    tracer_buffer.vertex_buffer.as_ref().unwrap(),
                    0,
                    bytemuck::cast_slice(&data),
                );
                let mut indices = Vec::default();
                for index in 0..num_of_shapes {
                    indices.extend(INDICES.iter().map(|i| i + (4 * index as u16)))
                }

                queue.write_buffer(
                    tracer_buffer.index_buffer.as_ref().unwrap(),
                    0,
                    bytemuck::cast_slice(&indices),
                );
            } else {
                let (v_buffer, i_buffer) = generate_vertex_and_indexes_buffer(&device, &data);
                if let Some(buffer) = &tracer_buffer.vertex_buffer {
                    buffer.destroy();
                }
                if let Some(buffer) = &tracer_buffer.index_buffer {
                    buffer.destroy();
                }
                tracer_buffer.vertex_buffer = Some(v_buffer);
                tracer_buffer.index_buffer = Some(i_buffer);

                tracer_buffer.size = num_of_shapes;
            }

            render_pass.set_pipeline(&tracer_pipeline);
            render_pass.set_bind_group(0, &camera_buffer.bind_group, &[]);

            render_pass
                .set_vertex_buffer(0, tracer_buffer.vertex_buffer.as_ref().unwrap().slice(..));
            render_pass.set_index_buffer(
                tracer_buffer.index_buffer.as_ref().unwrap().slice(..),
                wgpu::IndexFormat::Uint16,
            );

            render_pass.draw_indexed(0..num_of_shapes as u32 * 6, 0, 0..1);
        }
    }
}

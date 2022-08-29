use std::iter;

use crate::{ActiveCamera, Camera, CameraUniform, Color, SpriteType, TextureManager};
use wgpu::{
    util::DeviceExt, Adapter, BindGroup, BindGroupLayout, Buffer, Device, Instance, Queue,
    RenderPipeline, Surface,
};
use zengine_core::Transform;
use zengine_ecs::{
    system::{Commands, OptionalRes, OptionalUnsendableRes, Query, QueryIter, Res, UnsendableRes},
    Entity,
};
use zengine_macro::{Component, Resource, UnsendableResource};
use zengine_window::Window;

#[derive(Copy, Clone)]
pub enum CollisionTrace {
    Active,
    Inactive,
}

#[derive(Resource, Debug, Default)]
pub struct Background {
    pub color: Color,
}

#[derive(Component, Debug)]
pub struct Sprite<ST: SpriteType> {
    pub width: f32,
    pub height: f32,
    pub origin: glam::Vec3,
    pub color: Color,
    pub sprite_type: ST,
}

#[derive(UnsendableResource, Debug)]
pub struct RenderContext {
    surface: Surface,
    pub device: Device,
    pub queue: Queue,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    pub texture_bind_group_layout: BindGroupLayout,
    //camera_uniform: CameraUniform,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, 0.5, 0.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.0],
        tex_coords: [1.0, 0.0],
    },
];

const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

pub fn setup_render(window: OptionalUnsendableRes<Window>, mut commands: Commands) {
    // The instance is a handle to our GPU
    // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
    let window = window.expect("Cannot find a Window");
    let internal_window = &window.internal;

    let instance = Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(internal_window) };
    async fn create_adapter_device_queue(
        instance: &Instance,
        surface: &Surface,
    ) -> (Adapter, Device, Queue) {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                // Some(&std::path::Path::new("trace")), // Trace path
                None,
            )
            .await
            .unwrap();

        (adapter, device, queue)
    }

    let (adapter, device, queue) =
        pollster::block_on(create_adapter_device_queue(&instance, &surface));

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_supported_formats(&adapter)[0],
        width: window.width,
        height: window.height,
        present_mode: wgpu::PresentMode::Fifo,
    };
    surface.configure(&device, &config);

    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

    let camera_uniform = CameraUniform::default();
    // let camera_uniform = CameraUniform::new(
    //     &Camera {
    //         mode: CameraMode::Mode2D((3.0, 4.0)),
    //     },
    //     Some(&cgmath::Point3::new(0.0, 0.0, 50.0)),
    // );
    let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Camera Buffer"),
        contents: bytemuck::cast_slice(&[camera_uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let camera_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

    let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &camera_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: camera_buffer.as_entire_binding(),
        }],
        label: Some("camera_bind_group"),
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
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
                format: config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
            // or Features::POLYGON_MODE_POINT
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        // If the pipeline will be used with a multiview render pass, this
        // indicates how many array layers the attachments will have.
        multiview: None,
    });

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(INDICES),
        usage: wgpu::BufferUsages::INDEX,
    });

    commands.create_unsendable_resource(RenderContext {
        surface,
        device,
        queue,
        render_pipeline,
        vertex_buffer,
        index_buffer,
        texture_bind_group_layout,
        // camera_uniform,
        camera_buffer,
        camera_bind_group,
    });
}

fn pick_correct_camera<'a>(
    camera_query: &'a Query<(Entity, &Camera, &Transform)>,
    active_camera: &'a OptionalRes<ActiveCamera>,
) -> Option<(&'a Camera, &'a Transform)> {
    active_camera
        .as_ref()
        .map_or_else(
            || camera_query.iter().next(),
            |active| camera_query.iter().find(|(e, ..)| **e == active.entity),
        )
        .map(|(_, c, t)| (c, t))
}

pub fn renderer<SP: SpriteType>(
    render_context: OptionalUnsendableRes<RenderContext>,
    bg_color: Res<Background>,
    texture_manager: UnsendableRes<TextureManager<SP>>,
    camera_query: Query<(Entity, &Camera, &Transform)>,
    active_camera: OptionalRes<ActiveCamera>,
) {
    if let Some(texture) = texture_manager
        .textures
        .first()
        .and_then(|t| t.texture.as_ref())
        .map(|t| &t.diffuse_bind_group)
    {
        let render_context = render_context.unwrap();
        let output = render_context.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            render_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: bg_color.color.r,
                            g: bg_color.color.g,
                            b: bg_color.color.b,
                            a: bg_color.color.a,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&render_context.render_pipeline);
            render_pass.set_bind_group(0, texture, &[]);

            let camera_data = pick_correct_camera(&camera_query, &active_camera);
            if let Some((camera, transform)) = camera_data {
                render_context.queue.write_buffer(
                    &render_context.camera_buffer,
                    0,
                    bytemuck::cast_slice(&[CameraUniform::new(camera, Some(&transform.position))]),
                );
            } else {
                render_context.queue.write_buffer(
                    &render_context.camera_buffer,
                    0,
                    bytemuck::cast_slice(&[CameraUniform::default()]),
                );
            }
            render_pass.set_bind_group(1, &render_context.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, render_context.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                render_context.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
        }

        render_context.queue.submit(iter::once(encoder.finish()));
        output.present();
    }
}

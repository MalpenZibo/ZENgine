use std::iter;

use crate::{
    ActiveCamera, Camera, CameraUniform, Color, SpriteType, TextureHandleState, TextureManager,
};
use glam::{Mat4, Vec2, Vec3, Vec4};
use rustc_hash::FxHashMap;
use wgpu::{
    util::DeviceExt, Adapter, BindGroup, BindGroupLayout, Buffer, Device, Queue, RenderPipeline,
    Surface,
};
use zengine_core::Transform;
use zengine_ecs::{
    system::{
        Commands, OptionalRes, OptionalUnsendableRes, OptionalUnsendableResMut, Query, QueryIter,
        Res, UnsendableRes,
    },
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
    pub texture_bind_group_layout: BindGroupLayout,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    depth_pass: DepthPass,
}

#[derive(Debug)]
struct DepthPass {
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
    _sampler: wgpu::Sampler,
}

#[derive(UnsendableResource, Debug)]
pub struct VertexBuffers(FxHashMap<usize, (Buffer, usize, Buffer, usize)>);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 4],
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
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

fn create_depth_texture(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    label: &str,
) -> DepthPass {
    let size = wgpu::Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
    };
    let desc = wgpu::TextureDescriptor {
        label: Some(label),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
    };
    let texture = device.create_texture(&desc);
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        compare: Some(wgpu::CompareFunction::LessEqual),
        lod_min_clamp: -100.0,
        lod_max_clamp: 100.0,
        ..Default::default()
    });

    DepthPass {
        _texture: texture,
        view,
        _sampler: sampler,
    }
}

pub fn setup_render(window: OptionalUnsendableRes<Window>, mut commands: Commands) {
    // The instance is a handle to our GPU
    // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
    let window = window.expect("Cannot find a Window");
    let internal_window = &window.internal;

    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(internal_window) };
    async fn create_adapter_device_queue(
        instance: &wgpu::Instance,
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
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        // If the pipeline will be used with a multiview render pass, this
        // indicates how many array layers the attachments will have.
        multiview: None,
    });

    let depth_pass = create_depth_texture(&device, &config, "depth_texture");

    commands.create_unsendable_resource(RenderContext {
        surface,
        device,
        queue,
        render_pipeline,
        texture_bind_group_layout,
        camera_buffer,
        camera_bind_group,
        depth_pass,
    });
    commands.create_unsendable_resource(VertexBuffers(FxHashMap::default()));
}

fn pick_correct_camera<'a>(
    camera_query: &'a Query<(Entity, &Camera, Option<&Transform>)>,
    active_camera: &'a OptionalRes<ActiveCamera>,
) -> Option<(&'a Camera, Option<&'a Transform>)> {
    active_camera
        .as_ref()
        .map_or_else(
            || camera_query.iter().next(),
            |active| camera_query.iter().find(|(e, ..)| **e == active.entity),
        )
        .map(|(_, c, t)| (c, t))
}

fn calculate_vertices(
    width: f32,
    height: f32,
    origin: Vec3,
    relative_min: Vec2,
    relative_max: Vec2,
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
        },
        Vertex {
            position: transform
                .mul_vec4(Vec4::new(min_x, min_y, 0.0, 1.0))
                .to_array(),
            tex_coords: [min_u, max_v],
        },
        Vertex {
            position: transform
                .mul_vec4(Vec4::new(max_x, min_y, 0.0, 1.0))
                .to_array(),
            tex_coords: [max_u, max_v],
        },
        Vertex {
            position: transform
                .mul_vec4(Vec4::new(max_x, max_y, 0.0, 1.0))
                .to_array(),
            tex_coords: [max_u, min_v],
        },
    ]
}

fn generate_vertex_and_indexes_buffer(
    device: &Device,
    vertex: &Vec<Vertex>,
) -> (Buffer, usize, Buffer, usize) {
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

    (vertex_buffer, vertex.len(), index_buffer, indices.len())
}

pub fn renderer<ST: SpriteType>(
    render_context: OptionalUnsendableRes<RenderContext>,
    mut vertex_buffers: OptionalUnsendableResMut<VertexBuffers>,
    bg_color: Res<Background>,
    texture_manager: UnsendableRes<TextureManager<ST>>,
    camera_query: Query<(Entity, &Camera, Option<&Transform>)>,
    active_camera: OptionalRes<ActiveCamera>,
    sprite_query: Query<(&Sprite<ST>, &Transform)>,
) {
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
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &render_context.depth_pass.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(&render_context.render_pipeline);

        let camera_data = pick_correct_camera(&camera_query, &active_camera);
        if let Some((camera, transform)) = camera_data {
            render_context.queue.write_buffer(
                &render_context.camera_buffer,
                0,
                bytemuck::cast_slice(&[CameraUniform::new(camera, transform)]),
            );
        } else {
            render_context.queue.write_buffer(
                &render_context.camera_buffer,
                0,
                bytemuck::cast_slice(&[CameraUniform::default()]),
            );
        }

        let mut batched: FxHashMap<usize, Vec<Vertex>> = FxHashMap::default();
        for (s, t) in sprite_query.iter() {
            if let Some(sprite_handle) = texture_manager.get_sprite_handle(&s.sprite_type) {
                if texture_manager
                    .textures
                    .get(sprite_handle.texture_handle_index)
                    .unwrap()
                    .state
                    == TextureHandleState::Uploaded
                {
                    let batch = batched
                        .entry(sprite_handle.texture_handle_index)
                        .or_insert_with(Vec::default);

                    batch.extend(calculate_vertices(
                        s.width,
                        s.height,
                        s.origin,
                        sprite_handle.relative_min,
                        sprite_handle.relative_max,
                        t.get_transformation_matrix(),
                    ));
                }
            }
        }

        let vertex_buffers = &mut vertex_buffers.as_mut().unwrap().0;

        for (k, batch) in batched.iter() {
            let vertex_buffer = vertex_buffers.entry(*k).or_insert_with(|| {
                generate_vertex_and_indexes_buffer(&render_context.device, batch)
            });

            if vertex_buffer.1 < batch.len() {
                let new_buffer = generate_vertex_and_indexes_buffer(&render_context.device, batch);
                vertex_buffer.0.destroy();
                vertex_buffer.2.destroy();

                vertex_buffer.0 = new_buffer.0;
                vertex_buffer.1 = new_buffer.1;
                vertex_buffer.2 = new_buffer.2;
                vertex_buffer.3 = new_buffer.3;
            } else {
                render_context
                    .queue
                    .write_buffer(&vertex_buffer.0, 0, bytemuck::cast_slice(batch));
                let mut indices = Vec::default();
                for index in 0..batch.len() / 4 {
                    indices.extend(INDICES.iter().map(|i| i + (4 * index as u16)))
                }

                render_context.queue.write_buffer(
                    &vertex_buffer.2,
                    0,
                    bytemuck::cast_slice(&indices),
                );
            }
        }

        render_pass.set_bind_group(1, &render_context.camera_bind_group, &[]);
        for (k, (vertex, _, indices, i_size)) in vertex_buffers
            .iter()
            .filter(|(k, _)| batched.iter().any(|(b_k, _)| b_k == *k))
        {
            let texture = texture_manager
                .textures
                .get(*k)
                .and_then(|t| t.texture.as_ref())
                .unwrap();

            render_pass.set_bind_group(0, &texture.diffuse_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex.slice(..));
            render_pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..(*i_size) as u32, 0, 0..1);
        }
    }

    render_context.queue.submit(iter::once(encoder.finish()));
    output.present();
}

use std::iter;

use crate::{
    renderer_utils::{Vertex, INDICES},
    ActiveCamera, Background, Camera, CameraUniform, Color, Sprite, SpriteType, TextureHandleState,
    TextureManager,
};
use glam::{Mat4, Vec2, Vec3, Vec4};
use rustc_hash::FxHashMap;
use wgpu::{
    util::DeviceExt, Adapter, BindGroup, BindGroupLayout, Device, Queue, RenderPipeline, Surface,
};
use zengine_core::Transform;
use zengine_ecs::{
    system::{
        Commands, Local, OptionalRes, OptionalUnsendableRes, Query, QueryIter, Res, UnsendableRes,
    },
    Entity,
};
use zengine_macro::{Resource, UnsendableResource};
use zengine_window::Window;

#[derive(UnsendableResource, Debug)]
pub struct RenderContext {
    surface: Surface,
    pub device: Device,
    pub queue: Queue,
    render_pipeline: RenderPipeline,
    pub texture_bind_group_layout: BindGroupLayout,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: BindGroup,
}

#[derive(Resource, Default, Debug)]
pub struct Buffers(Vec<Option<Buffer>>);

#[derive(Debug)]
struct Buffer {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertex_size: usize,
    index_size: usize,
    ref_texture: usize,
}

#[derive(Debug)]
pub struct BatchLayer {
    z: f32,
    batches: FxHashMap<usize, Vec<Vertex>>,
}

#[derive(Resource, Default, Debug)]
pub struct BatcheLayers(Vec<BatchLayer>);

impl BatcheLayers {
    fn clear(&mut self) {
        for b_l in self.0.iter_mut() {
            for v in b_l.batches.values_mut() {
                v.clear();
            }
        }
    }
}

pub fn setup_render(window: OptionalUnsendableRes<Window>, mut commands: Commands) {
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

    commands.create_unsendable_resource(RenderContext {
        surface,
        device,
        queue,
        render_pipeline,
        texture_bind_group_layout,
        camera_buffer,
        camera_bind_group,
    });
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
    device: &Device,
    vertex: &Vec<Vertex>,
) -> (wgpu::Buffer, usize, wgpu::Buffer, usize) {
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

#[allow(clippy::too_many_arguments)]
pub fn renderer<ST: SpriteType>(
    render_context: OptionalUnsendableRes<RenderContext>,
    bg_color: Res<Background>,
    texture_manager: UnsendableRes<TextureManager<ST>>,
    camera_query: Query<(Entity, &Camera, Option<&Transform>)>,
    active_camera: OptionalRes<ActiveCamera>,
    sprite_query: Query<(&Sprite<ST>, &Transform)>,
    batches: Local<BatcheLayers>,
    vertex_buffers: Local<Buffers>,
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
                        r: bg_color.color.r as f64,
                        g: bg_color.color.g as f64,
                        b: bg_color.color.b as f64,
                        a: bg_color.color.a as f64,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
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

        batches.clear();
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

                    let batch_layer = match batches.0.binary_search_by(|l| l.z.total_cmp(&z)) {
                        Ok(index) => batches.0.get_mut(index).unwrap(),
                        Err(index) => {
                            batches.0.insert(
                                index,
                                BatchLayer {
                                    z,
                                    batches: FxHashMap::default(),
                                },
                            );
                            batches.0.get_mut(index).unwrap()
                        }
                    };

                    match batch_layer
                        .batches
                        .get_mut(&sprite_handle.texture_handle_index)
                    {
                        Some(batch) => {
                            batch.extend(calculate_vertices(
                                s.width,
                                s.height,
                                s.origin,
                                sprite_handle.relative_min,
                                sprite_handle.relative_max,
                                &s.color,
                                t.get_transformation_matrix(),
                            ));
                        }
                        None => {
                            batch_layer.batches.insert(
                                sprite_handle.texture_handle_index,
                                calculate_vertices(
                                    s.width,
                                    s.height,
                                    s.origin,
                                    sprite_handle.relative_min,
                                    sprite_handle.relative_max,
                                    &s.color,
                                    t.get_transformation_matrix(),
                                )
                                .into(),
                            );
                        }
                    }
                }
            }
        }

        let vertex_buffers = &mut vertex_buffers.0;
        let buffer_used = batches.0.iter().map(|l| l.batches.len()).sum();
        vertex_buffers.extend(
            (vertex_buffers.len()..buffer_used)
                .into_iter()
                .map(|_| None),
        );

        let mut buffer_iter = vertex_buffers.iter_mut();

        for batch_layer in batches.0.iter().rev() {
            for (k, v) in batch_layer.batches.iter() {
                let buffer = buffer_iter.next();
                if let Some(Some(buffer)) = buffer {
                    if buffer.vertex_size < v.len() {
                        let (v_buffer, v_size, i_buffer, i_size) =
                            generate_vertex_and_indexes_buffer(&render_context.device, v);
                        buffer.vertex_buffer.destroy();
                        buffer.index_buffer.destroy();

                        buffer.vertex_buffer = v_buffer;
                        buffer.vertex_size = v_size;
                        buffer.index_buffer = i_buffer;
                        buffer.index_size = i_size;

                        buffer.ref_texture = *k;
                    } else {
                        render_context.queue.write_buffer(
                            &buffer.vertex_buffer,
                            0,
                            bytemuck::cast_slice(v),
                        );
                        let mut indices = Vec::default();
                        for index in 0..v.len() / 4 {
                            indices.extend(INDICES.iter().map(|i| i + (4 * index as u16)))
                        }

                        render_context.queue.write_buffer(
                            &buffer.index_buffer,
                            0,
                            bytemuck::cast_slice(&indices),
                        );

                        buffer.ref_texture = *k;
                    }
                } else if let Some(buffer @ None) = buffer {
                    let (v_buffer, v_size, i_buffer, i_size) =
                        generate_vertex_and_indexes_buffer(&render_context.device, v);
                    buffer.replace(Buffer {
                        vertex_buffer: v_buffer,
                        vertex_size: v_size,
                        index_buffer: i_buffer,
                        index_size: i_size,
                        ref_texture: *k,
                    });
                }
            }
        }

        render_pass.set_bind_group(1, &render_context.camera_bind_group, &[]);
        for b in vertex_buffers.iter().take(buffer_used) {
            let b = b.as_ref().unwrap();
            let texture = texture_manager
                .textures
                .get(b.ref_texture)
                .and_then(|t1| t1.texture.as_ref())
                .unwrap();

            render_pass.set_bind_group(0, &texture.diffuse_bind_group, &[]);
            render_pass.set_vertex_buffer(0, b.vertex_buffer.slice(..));
            render_pass.set_index_buffer(b.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..b.index_size as u32, 0, 0..1);
        }
    }

    render_context.queue.submit(iter::once(encoder.finish()));
    output.present();
}

use std::iter;

use crate::{
    renderer_utils::{Vertex, INDICES},
    ActiveCamera, Background, Camera, CameraUniform, Color, Sprite, SpriteHandle, SpriteType,
    TextureHandleState, TextureManager,
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
pub struct Buffer {
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

#[allow(clippy::too_many_arguments)]
pub fn renderer<ST: SpriteType>(
    render_context: OptionalUnsendableRes<RenderContext>,
    bg_color: Res<Background>,
    texture_manager: UnsendableRes<TextureManager<ST>>,
    camera_query: Query<(Entity, &Camera, Option<&Transform>)>,
    active_camera: OptionalRes<ActiveCamera>,
    sprite_query: Query<(&Sprite<ST>, &Transform)>,
    vertex_buffer: Local<Buffer>,
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
        if vertex_buffer.size >= num_of_sprite && vertex_buffer.vertex_buffer.is_some() {
            render_context.queue.write_buffer(
                vertex_buffer.vertex_buffer.as_ref().unwrap(),
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

            render_context.queue.write_buffer(
                vertex_buffer.index_buffer.as_ref().unwrap(),
                0,
                bytemuck::cast_slice(&indices),
            );
        } else {
            let (v_buffer, i_buffer) = generate_vertex_and_indexes_buffer(
                &render_context.device,
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
            if let Some(buffer) = &vertex_buffer.vertex_buffer {
                buffer.destroy();
            }
            if let Some(buffer) = &vertex_buffer.index_buffer {
                buffer.destroy();
            }
            vertex_buffer.vertex_buffer = Some(v_buffer);
            vertex_buffer.index_buffer = Some(i_buffer);

            vertex_buffer.size = num_of_sprite;
        }

        render_pass.set_bind_group(1, &render_context.camera_bind_group, &[]);

        render_pass.set_vertex_buffer(0, vertex_buffer.vertex_buffer.as_ref().unwrap().slice(..));
        render_pass.set_index_buffer(
            vertex_buffer.index_buffer.as_ref().unwrap().slice(..),
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

    render_context.queue.submit(iter::once(encoder.finish()));
    output.present();
}

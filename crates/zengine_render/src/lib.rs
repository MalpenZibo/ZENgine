use std::iter;

use wgpu::{Adapter, Device, Instance, Queue, RenderPipeline, Surface};
use zengine_ecs::{
    system::{Commands, OptionalUnsendableRes},
    UnsendableResource,
};
use zengine_graphic::{Color, SpriteType};
use zengine_macro::{Component, Resource};
//use zengine_math::Vec3;
use zengine_window::Window;

// mod gl_utilities;
// mod render_system;

// pub use render_system::{render_system, setup_render};

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
    //pub origin: Vector3,
    pub color: Color,
    pub sprite_type: ST,
}

#[derive(Debug)]
pub struct RenderContext {
    surface: Surface,
    device: Device,
    queue: Queue,
    render_pipeline: RenderPipeline,
}

impl UnsendableResource for RenderContext {}

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

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
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

    commands.create_unsendable_resource(RenderContext {
        surface,
        device,
        queue,
        render_pipeline,
    });
}

pub fn renderer(render_context: OptionalUnsendableRes<RenderContext>) {
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
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&render_context.render_pipeline);
        render_pass.draw(0..3, 0..1);
    }

    render_context.queue.submit(iter::once(encoder.finish()));
    output.present();
}

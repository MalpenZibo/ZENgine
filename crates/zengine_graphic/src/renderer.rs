use std::{
    iter,
    ops::{Deref, DerefMut},
};

use crate::Background;
use wgpu::{Adapter, BindGroupLayout, Surface, SurfaceConfiguration};
use zengine_ecs::system::{Commands, OptionalRes, OptionalUnsendableRes, Res, ResMut};
use zengine_macro::Resource;
use zengine_window::Window;

#[derive(Resource, Debug)]
pub struct SurfaceData {
    pub surface_config: SurfaceConfiguration,
    pub surface: Surface,
}

#[derive(Resource, Debug)]
pub struct TextureBindGroupLayout(BindGroupLayout);

impl Deref for TextureBindGroupLayout {
    type Target = BindGroupLayout;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Resource, Debug)]
pub struct Queue(wgpu::Queue);

impl Deref for Queue {
    type Target = wgpu::Queue;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Resource, Debug)]
pub struct Device(wgpu::Device);

impl Deref for Device {
    type Target = wgpu::Device;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct RenderContext {
    pub surface_texture: wgpu::SurfaceTexture,
    pub texture_view: wgpu::TextureView,
    pub command_encoder: wgpu::CommandEncoder,
}

#[derive(Resource, Default, Debug)]
pub struct RenderContextInstance(Option<RenderContext>);

impl Deref for RenderContextInstance {
    type Target = Option<RenderContext>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RenderContextInstance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
    ) -> (Adapter, wgpu::Device, wgpu::Queue) {
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

    commands.create_resource(SurfaceData {
        surface_config: config,
        surface,
    });
    commands.create_resource(TextureBindGroupLayout(texture_bind_group_layout));
    commands.create_resource(Queue(queue));
    commands.create_resource(Device(device));
}

pub fn clear(
    surface_data: OptionalRes<SurfaceData>,
    device: OptionalRes<Device>,
    mut render_context: ResMut<RenderContextInstance>,
    bg_color: Res<Background>,
) {
    if let (Some(surface_data), Some(device)) = (surface_data, device) {
        let surface_texture = surface_data.surface.get_current_texture().unwrap();
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        render_context.replace(RenderContext {
            texture_view,
            surface_texture,
            command_encoder,
        });
        let render_context = render_context.as_mut().unwrap();
        {
            render_context
                .command_encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Clear Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &render_context.texture_view,
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
        }
    }
}

pub fn present(queue: OptionalRes<Queue>, mut render_context: ResMut<RenderContextInstance>) {
    let render_context = render_context.take().unwrap();

    queue
        .unwrap()
        .submit(iter::once(render_context.command_encoder.finish()));
    render_context.surface_texture.present();
}

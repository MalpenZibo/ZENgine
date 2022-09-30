use crate::Background;
use std::{
    iter,
    ops::{Deref, DerefMut},
};
use wgpu::{BindGroupLayout, SurfaceConfiguration};
use zengine_ecs::system::{Commands, OptionalRes, OptionalUnsendableRes, Res, ResMut};
use zengine_macro::Resource;
use zengine_window::{Window, WindowSpecs};

#[derive(Resource, Debug)]
pub struct TextureBindGroupLayout(BindGroupLayout);

impl Deref for TextureBindGroupLayout {
    type Target = BindGroupLayout;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Resource, Default, Debug)]
pub struct Surface(Option<(SurfaceConfiguration, wgpu::Surface, usize)>);

#[derive(Debug)]
pub enum SurfaceError {
    NotInitialize,
    NoMoreValid,
}

impl Surface {
    pub fn get_surface_texture(
        &self,
        windows_specs: &WindowSpecs,
    ) -> Result<wgpu::SurfaceTexture, SurfaceError> {
        self.0
            .as_ref()
            .ok_or(SurfaceError::NotInitialize)
            .and_then(|s| {
                if s.2 != windows_specs.surface_id {
                    Err(SurfaceError::NoMoreValid)
                } else {
                    s.1.get_current_texture()
                        .map_err(|_| SurfaceError::NoMoreValid)
                }
            })
    }

    pub fn get_config(&self) -> Option<&SurfaceConfiguration> {
        self.0.as_ref().map(|s| &s.0)
    }

    fn create_surface(
        &mut self,
        window: &Window,
        instance: &Instance,
        adapter: &Adapter,
        device: &Device,
        window_specs: &WindowSpecs,
    ) {
        let internal_window = &window.internal;

        let surface = unsafe { instance.create_surface(internal_window) };
        log::warn!("create surface");
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(adapter)[0],
            width: window_specs.size.x,
            height: window_specs.size.y,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(device, &config);

        self.0.replace((config, surface, window_specs.surface_id));
    }
}

#[derive(Resource, Debug)]
pub struct Instance(wgpu::Instance);

impl Deref for Instance {
    type Target = wgpu::Instance;
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

#[derive(Resource, Debug)]
pub struct Adapter(wgpu::Adapter);

impl Deref for Adapter {
    type Target = wgpu::Adapter;
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

pub fn setup_render(
    window: OptionalUnsendableRes<Window>,
    window_specs: Res<WindowSpecs>,
    mut commands: Commands,
) {
    let window = window.expect("Cannot find a Window");
    let internal_window = &window.internal;

    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(internal_window) };
    async fn create_adapter_device_queue(
        instance: &wgpu::Instance,
        surface: &wgpu::Surface,
    ) -> (wgpu::Adapter, wgpu::Device, wgpu::Queue) {
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
        width: window_specs.size.x,
        height: window_specs.size.y,
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

    commands.create_resource(TextureBindGroupLayout(texture_bind_group_layout));
    commands.create_resource(Instance(instance));
    commands.create_resource(Queue(queue));
    commands.create_resource(Device(device));
    commands.create_resource(Adapter(adapter));
    commands.create_resource(Surface(Some((config, surface, window_specs.surface_id))));
}

#[allow(clippy::too_many_arguments)]
pub fn clear(
    window: OptionalUnsendableRes<Window>,
    window_specs: Res<WindowSpecs>,
    device: OptionalRes<Device>,
    instance: OptionalRes<Instance>,
    adapter: OptionalRes<Adapter>,
    mut surface: ResMut<Surface>,
    mut render_context: ResMut<RenderContextInstance>,
    bg_color: Res<Background>,
) {
    if let (Some(window), Some(device), Some(instance), Some(adapter)) =
        (window, device, instance, adapter)
    {
        let surface_texture = match surface.get_surface_texture(&window_specs) {
            Ok(surface_texture) => surface_texture,
            Err(_) => {
                surface.create_surface(&window, &instance, &adapter, &device, &window_specs);

                surface
                    .get_surface_texture(&window_specs)
                    .expect("Couldn't initialize a surface")
            }
        };

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
    if let Some(render_context) = render_context.take() {
        queue
            .unwrap()
            .submit(iter::once(render_context.command_encoder.finish()));
        render_context.surface_texture.present();
    }
}

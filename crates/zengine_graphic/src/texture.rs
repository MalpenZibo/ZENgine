use crate::{Device, Image, Queue, TextureBindGroupLayout};
use glam::Vec2;
use zengine_asset::{AssetEvent, Assets, Handle};
use zengine_ecs::system::{EventStream, Res, ResMut};
use zengine_macro::Asset;

#[derive(Debug)]
pub(crate) struct GpuImage {
    pub texture: wgpu::Texture,
    pub _view: wgpu::TextureView,
    pub _sampler: wgpu::Sampler,
    pub diffuse_bind_group: wgpu::BindGroup,
}

impl Drop for GpuImage {
    fn drop(&mut self) {
        self.texture.destroy();
    }
}

/// [Asset](zengine_asset::Asset) that rappresent a simple Texture
#[derive(Asset, Debug)]
pub struct Texture {
    image: Handle<Image>,
    pub(crate) gpu_image: Option<GpuImage>,
    pub size: Vec2,
    pub ratio: f32,
}

impl Texture {
    fn new(image: Handle<Image>) -> Self {
        Self {
            image,
            gpu_image: None,
            size: Vec2::ZERO,
            ratio: 1.,
        }
    }

    pub(crate) fn convert_to_gpu_image(
        &mut self,
        device: &Device,
        queue: &Queue,
        texture_bind_group_layout: &TextureBindGroupLayout,
        images: &Assets<Image>,
    ) {
        if let Some(image) = images.get(&self.image) {
            let size = wgpu::Extent3d {
                width: image.width,
                height: image.height,
                depth_or_array_layers: 1,
            };
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("diffuse_texture"),
                view_formats: &[],
            });

            queue.write_texture(
                wgpu::ImageCopyTexture {
                    aspect: wgpu::TextureAspect::All,
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                &image.data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * image.width),
                    rows_per_image: Some(image.height),
                },
                size,
            );

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });

            let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
                label: Some("diffuse_bind_group"),
            });

            self.image = self.image.as_weak();
            self.gpu_image = Some(GpuImage {
                texture,
                _view: view,
                _sampler: sampler,
                diffuse_bind_group,
            });
            self.size = Vec2::new(image.width as f32, image.height as f32);
            self.ratio = image.width as f32 / image.height as f32;
        }
    }
}

/// Add functionalities to create a [Texture] to the [`Assets<Texture>`] storage
pub trait TextureAssets {
    /// Creates a [Texture] asset returning a strong [Handle] to it with the given Image handle
    fn create_texture(&mut self, image: &Handle<Image>) -> Handle<Texture>;
}

impl TextureAssets for Assets<Texture> {
    fn create_texture(&mut self, image_handle: &Handle<Image>) -> Handle<Texture> {
        let handle = Handle::weak(image_handle.get_id().clone_with_different_type::<Texture>());
        self.set(handle, Texture::new(image_handle.clone()))
    }
}

pub(crate) fn prepare_texture_asset(
    texture_bind_group_layout: Option<Res<TextureBindGroupLayout>>,
    device: Option<Res<Device>>,
    queue: Option<Res<Queue>>,
    textures: Option<ResMut<Assets<Texture>>>,
    images: Option<Res<Assets<Image>>>,
    images_asset_event: EventStream<AssetEvent<Image>>,
) {
    let events = images_asset_event.read();
    if let (
        Some(mut textures),
        Some(images),
        Some(device),
        Some(queue),
        Some(texture_bind_group_layout),
    ) = (textures, images, device, queue, texture_bind_group_layout)
    {
        for e in events {
            if let AssetEvent::Loaded(handle) = e {
                let handle = Handle::weak(handle.get_id().clone_with_different_type::<Texture>());
                if let Some(texture) = textures.get_mut(&handle) {
                    texture.convert_to_gpu_image(
                        &device,
                        &queue,
                        &texture_bind_group_layout,
                        &images,
                    )
                }
            }
        }
    }
}

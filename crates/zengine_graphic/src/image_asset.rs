use std::num::NonZeroU32;

use crate::{Device, Queue, TextureBindGroupLayout};
use image::{DynamicImage, GenericImageView};
use zengine_asset::{AssetEvent, AssetLoader, Assets};
use zengine_ecs::system::{EventStream, OptionalRes, OptionalResMut};
use zengine_macro::Asset;

#[derive(Debug)]
pub struct GpuImage {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub diffuse_bind_group: wgpu::BindGroup,
}

impl Drop for GpuImage {
    fn drop(&mut self) {
        self.texture.destroy();
    }
}

#[derive(Asset, Default, Debug)]
pub struct Image {
    pub width: u32,
    pub height: u32,

    pub data: Option<Vec<u8>>,
    pub gpu_image: Option<GpuImage>,
}

impl Image {
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            data: Some(data),
            ..Default::default()
        }
    }

    pub fn convert_to_gpu_image(
        &mut self,
        device: &Device,
        queue: &Queue,
        texture_bind_group_layout: &TextureBindGroupLayout,
    ) {
        if let Some(data) = &self.data {
            let size = wgpu::Extent3d {
                width: self.width,
                height: self.height,
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
            });

            queue.write_texture(
                wgpu::ImageCopyTexture {
                    aspect: wgpu::TextureAspect::All,
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(4 * self.width),
                    rows_per_image: NonZeroU32::new(self.height),
                },
                size,
            );

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
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

            self.data = None;
            self.gpu_image = Some(GpuImage {
                texture,
                view,
                sampler,
                diffuse_bind_group,
            })
        }
    }
}

#[derive(Debug)]
pub struct ImageLoader;

impl AssetLoader for ImageLoader {
    fn extension(&self) -> &[&str] {
        &["png", "jpg", "jpeg", "bmp"]
    }

    fn load(&self, data: Vec<u8>, context: &mut zengine_asset::LoaderContext) {
        let img =
            image::load_from_memory(&data).unwrap_or_else(|e| panic!("Could not load image {}", e));

        let (width, height) = img.dimensions();

        let img = match img {
            DynamicImage::ImageRgba8(img) => img,
            img => img.to_rgba8(),
        };

        context.set_asset(Image {
            width,
            height,
            data: Some(img.into_raw()),
            gpu_image: None,
        })
    }
}

pub fn prepare_image_asset(
    texture_bind_group_layout: OptionalRes<TextureBindGroupLayout>,
    device: OptionalRes<Device>,
    queue: OptionalRes<Queue>,
    images: OptionalResMut<Assets<Image>>,
    image_asset_event: EventStream<AssetEvent<Image>>,
) {
    let events = image_asset_event.read();
    if let (Some(mut images), Some(device), Some(queue), Some(texture_bind_group_layout)) =
        (images, device, queue, texture_bind_group_layout)
    {
        for e in events {
            if let AssetEvent::Loaded(handle) = e {
                if let Some(image) = images.get_mut(handle) {
                    image.convert_to_gpu_image(&device, &queue, &texture_bind_group_layout);
                }
            }
        }
    }
}

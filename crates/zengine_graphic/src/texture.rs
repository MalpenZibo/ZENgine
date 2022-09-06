use glam::Vec2;
use rustc_hash::FxHashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::{any::Any, num::NonZeroU32};
use wgpu::BindGroupLayout;
use zengine_asset::image_loader::{self, ImageAsset, ImageAssetHandler};
use zengine_ecs::system::{OptionalRes, UnsendableResMut};
use zengine_ecs::UnsendableResource;

use crate::{Device, Queue, TextureBindGroupLayout};

pub trait SpriteType: Any + Eq + PartialEq + Hash + Clone + Debug + Send + Sync {}
impl SpriteType for String {}

#[derive(Debug)]
pub struct SpriteDescriptor {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Eq, PartialEq, Debug)]
pub enum TextureHandleState {
    ToUpload(Vec<u8>),
    Uploaded,
    ToUnload,
}

#[derive(Debug)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub diffuse_bind_group: wgpu::BindGroup,
}

#[derive(Debug)]
pub struct SpriteHandle {
    pub relative_min: Vec2,
    pub relative_max: Vec2,
    pub width: u32,
    pub height: u32,
    pub texture_handle_index: usize,
}

#[derive(Debug)]
pub struct TextureHandle {
    pub width: u32,
    pub height: u32,
    pub state: TextureHandleState,
    pub texture: Option<Texture>,
}

pub struct TextureLoader<'a, ST: SpriteType> {
    file_path: String,
    texture_manager: &'a mut TextureManager<ST>,
    sprites: FxHashMap<ST, SpriteDescriptor>,
}

impl<'a, ST: SpriteType> TextureLoader<'a, ST> {
    pub fn with_sprite(mut self, sprite_type: ST, descriptor: SpriteDescriptor) -> Self {
        self.sprites.insert(sprite_type, descriptor);

        self
    }

    pub fn load(self) {
        self.texture_manager.load(&self.file_path, self.sprites)
    }
}

#[derive(Debug)]
pub struct Image<ST: SpriteType> {
    image_asset: ImageAssetHandler,
    image: Option<ImageAsset>,
    sprite_descriptor: FxHashMap<ST, SpriteDescriptor>,
}

#[derive(Debug)]
pub struct TextureManager<ST: SpriteType> {
    images: Vec<Image<ST>>,
    pub sprites: FxHashMap<ST, SpriteHandle>,
    pub textures: Vec<TextureHandle>,
}
impl<ST: SpriteType> UnsendableResource for TextureManager<ST> {}

impl<ST: SpriteType> Default for TextureManager<ST> {
    fn default() -> Self {
        TextureManager {
            images: Vec::default(),
            sprites: FxHashMap::default(),
            textures: Vec::default(),
        }
    }
}

impl<ST: SpriteType> TextureManager<ST> {
    pub(self) fn load(&mut self, file_path: &str, sprites: FxHashMap<ST, SpriteDescriptor>) {
        let img = image_loader::load(file_path);
        self.images.push(Image {
            sprite_descriptor: sprites,
            image_asset: img,
            image: None,
        });
    }

    pub fn handle_loaded_image(&mut self) {
        let mut indices = Vec::<usize>::new();

        for (index, i) in self.images.iter_mut().enumerate() {
            if let Ok(img) = i.image_asset.try_recv() {
                i.image = Some(img);
                indices.push(index);
            }
        }

        for i in indices.iter().rev() {
            let image = self.images.swap_remove(*i);
            if let Some(img) = image.image {
                let texture_index = self.textures.len();
                self.textures.push(TextureHandle {
                    width: img.width,
                    height: img.height,
                    state: TextureHandleState::ToUpload(img.data),
                    texture: None,
                });

                for (sprite_type, descriptor) in image.sprite_descriptor {
                    let sprite_handle = SpriteHandle {
                        relative_min: Vec2::new(
                            descriptor.x as f32 / img.width as f32,
                            descriptor.y as f32 / img.height as f32,
                        ),
                        relative_max: Vec2::new(
                            (descriptor.x + descriptor.width) as f32 / img.width as f32,
                            (descriptor.y + descriptor.height) as f32 / img.height as f32,
                        ),
                        width: descriptor.width,
                        height: descriptor.height,
                        texture_handle_index: texture_index,
                    };

                    if let Some(old_sprite) =
                        self.sprites.insert(sprite_type.clone(), sprite_handle)
                    {
                        if !self.texture_still_used(old_sprite.texture_handle_index) {
                            self.set_texture_to_unload(old_sprite.texture_handle_index);
                        }
                    }
                }
            }
        }
    }

    pub fn create(&mut self, file_path: &str) -> TextureLoader<ST> {
        TextureLoader {
            texture_manager: self,
            file_path: file_path.to_string(),
            sprites: FxHashMap::default(),
        }
    }

    pub fn destroy(&mut self, sprite_type: ST) {
        if let Some(sprite) = self.sprites.remove(&sprite_type) {
            if self.texture_still_used(sprite.texture_handle_index) {
                self.set_texture_to_unload(sprite.texture_handle_index);
            }
        }
    }

    pub fn get_sprite_handle(&self, sprite_type: &ST) -> Option<&SpriteHandle> {
        self.sprites.get(sprite_type)
    }

    fn texture_still_used(&self, texture_handle_index: usize) -> bool {
        self.sprites
            .values()
            .any(|sprite| sprite.texture_handle_index == texture_handle_index)
    }

    fn set_texture_to_unload(&mut self, texture_handle_index: usize) {
        if let Some(texture_handle) = self.textures.get_mut(texture_handle_index) {
            texture_handle.state = TextureHandleState::ToUnload;
        }
    }

    pub fn upload(
        &mut self,
        device: &Device,
        queue: &Queue,
        texture_bind_group_layout: &BindGroupLayout,
    ) {
        for t in self
            .textures
            .iter_mut()
            .filter(|t| matches!(t.state, TextureHandleState::ToUpload(_)))
        {
            if let TextureHandleState::ToUpload(image) = &t.state {
                let size = wgpu::Extent3d {
                    width: t.width,
                    height: t.height,
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
                    image,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: NonZeroU32::new(4 * t.width),
                        rows_per_image: NonZeroU32::new(t.height),
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

                t.state = TextureHandleState::Uploaded;
                t.texture = Some(Texture {
                    texture,
                    view,
                    sampler,
                    diffuse_bind_group,
                })
            }
        }
    }

    pub fn unload(&mut self) {
        for t in self
            .textures
            .iter_mut()
            .filter(|t| matches!(t.state, TextureHandleState::ToUnload))
        {
            t.texture
                .as_ref()
                .expect("Cannot find the wgpu::Texture")
                .texture
                .destroy();
        }

        for index in self
            .textures
            .iter()
            .enumerate()
            .filter_map(|(index, t)| {
                if matches!(t.state, TextureHandleState::ToUnload) {
                    Some(index)
                } else {
                    None
                }
            })
            .collect::<Vec<usize>>()
        {
            let last_index = self.textures.len() - 1;
            self.textures.swap_remove(index);

            if index != last_index {
                for s in self
                    .sprites
                    .iter_mut()
                    .filter(|(_, s)| s.texture_handle_index == last_index)
                {
                    s.1.texture_handle_index = index;
                }
            }
        }
    }
}

pub fn texture_loader<ST: SpriteType>(
    texture_bind_group_layout: OptionalRes<TextureBindGroupLayout>,
    device: OptionalRes<Device>,
    queue: OptionalRes<Queue>,
    mut texture_manager: UnsendableResMut<TextureManager<ST>>,
) {
    texture_manager.handle_loaded_image();

    if let (Some(device), Some(queue), Some(texture_bind_group_layout)) =
        (device, queue, texture_bind_group_layout)
    {
        texture_manager.upload(&device, &queue, &texture_bind_group_layout);

        texture_manager.unload();
    }
}

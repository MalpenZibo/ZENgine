use crate::{Device, Image, Queue, Texture, TextureAssets, TextureBindGroupLayout};
use glam::Vec2;
use rectangle_pack::{
    contains_smallest_box, pack_rects, volume_heuristic, GroupedRectsToPlace, PackedLocation,
    RectToInsert, RectanglePackError, TargetBin,
};
use rustc_hash::FxHashMap;
use std::{
    collections::BTreeMap,
    hash::{Hash, Hasher},
};
use zengine_asset::{AssetEvent, Assets, Handle, HandleId};
use zengine_ecs::system::{EventStream, Res, ResMut};
use zengine_macro::Asset;

const BYTE_PER_PIXEL: usize = 4;
const PADDING: u32 = 1;

#[derive(Debug)]
pub(crate) struct ImageRect {
    pub(crate) relative_min: Vec2,
    pub(crate) relative_max: Vec2,
}

/// [Asset](zengine_asset::Asset) that rappresent a Texture Atlas,
/// a Texture that contains many images
#[derive(Asset, Debug)]
pub struct TextureAtlas {
    width: u32,
    height: u32,
    images: FxHashMap<Handle<Image>, bool>,
    image_handles: FxHashMap<Handle<Image>, usize>,
    image_rects: Vec<ImageRect>,
    pub(crate) texture: Option<Handle<Texture>>,
}

impl TextureAtlas {
    pub(crate) fn get_rect(&self, image_handle: &Handle<Image>) -> &ImageRect {
        self.image_rects
            .get(*self.image_handles.get(image_handle).unwrap())
            .unwrap()
    }

    fn finalized(&self) -> bool {
        self.texture.is_some()
    }

    fn set_image_loaded(&mut self, image_handle: &Handle<Image>) {
        if let Some(image) = self.images.get_mut(image_handle) {
            *image = true;
        }
    }

    fn all_images_loaded(&self) -> bool {
        !self.images.iter().any(|(_, flag)| !flag)
    }

    fn finalize_atlas(
        &mut self,
        images: &mut Assets<Image>,
        textures: &mut Assets<Texture>,
        device: &Device,
        queue: &Queue,
        texture_bind_group_layout: &TextureBindGroupLayout,
    ) {
        let mut width = 256;
        let mut height = 256;

        let mut placements = None;
        let mut final_image = Image::default();

        let mut rects_to_place: GroupedRectsToPlace<Handle<Image>> = GroupedRectsToPlace::new();
        for (handle, image) in self
            .images
            .keys()
            .map(|handle| (handle, images.get(handle).unwrap()))
        {
            rects_to_place.push_rect(
                handle.clone_as_weak(),
                None,
                RectToInsert::new(image.width + PADDING * 2, image.height + PADDING * 2, 1),
            );
        }

        while placements.is_none() {
            let mut containers = BTreeMap::new();
            containers.insert(0, TargetBin::new(width, height, 1));

            placements = match pack_rects(
                &rects_to_place,
                &mut containers,
                &volume_heuristic,
                &contains_smallest_box,
            ) {
                Ok(placements) => {
                    final_image = Image::new(
                        width,
                        height,
                        vec![0; BYTE_PER_PIXEL * (width * height) as usize],
                    );

                    Some(placements)
                }
                Err(RectanglePackError::NotEnoughBinSpace) => {
                    if width < height {
                        width *= 2;
                    } else {
                        height *= 2;
                    }
                    None
                }
            }
        }

        self.width = width;
        self.height = height;

        let placements = placements.unwrap();
        self.image_rects = Vec::with_capacity(placements.packed_locations().len());
        for (image_handle, (_, location)) in placements.packed_locations().iter() {
            let image = images.get(image_handle).unwrap();

            let padding = Vec2::new(PADDING as f32, PADDING as f32);

            let min = Vec2::new(location.x() as f32, location.y() as f32) + padding;
            let max =
                min + Vec2::new(location.width() as f32, location.height() as f32) - (padding * 2.);

            self.image_handles
                .insert(image_handle.clone_as_weak(), self.image_rects.len());

            let relative_min = Vec2::new(min.x / width as f32, min.y / height as f32);
            let relative_max = Vec2::new(max.x / width as f32, max.y / height as f32);

            self.image_rects.push(ImageRect {
                relative_min,
                relative_max,
            });

            Self::copy_image_to_atlas(&mut final_image, image, location);
        }

        let texture_handle = textures.create_texture(&images.add(final_image));
        if let Some(texture) = textures.get_mut(&texture_handle) {
            texture.convert_to_gpu_image(device, queue, texture_bind_group_layout, images);
        }

        self.images.clear();
        self.images.shrink_to(0);
        self.texture = Some(texture_handle);
    }

    fn copy_image_to_atlas(atlas_image: &mut Image, image: &Image, location: &PackedLocation) {
        let source_width = (location.width() - PADDING * 2) as usize;
        let source_height = (location.height() - PADDING * 2) as usize;

        let target_width = atlas_image.width as usize;

        let x = (location.x() + PADDING) as usize;
        let y = (location.y() + PADDING) as usize;

        let target_data = &mut atlas_image.data;
        let source_data = &image.data;

        for (source_row, target_row) in (y..y + source_height).enumerate() {
            let target_begin = (target_row * target_width + x) * BYTE_PER_PIXEL;
            let target_end = target_begin + source_width * BYTE_PER_PIXEL;

            let source_begin = source_row * source_width * BYTE_PER_PIXEL;
            let source_end = source_begin + source_width * BYTE_PER_PIXEL;

            target_data[target_begin..target_end]
                .copy_from_slice(&source_data[source_begin..source_end])
        }
    }
}

/// Add functionalities to create a [TextureAtlas] to the [Assets<TextureAtlas>] storage
pub trait TextureAtlasAssets {
    /// Creates a [TextureAtlas] asset returning a strong [Handle] to it with the given Images handle
    fn create_texture_atlas(&mut self, images: &[&Handle<Image>]) -> Handle<TextureAtlas>;
}

impl TextureAtlasAssets for Assets<TextureAtlas> {
    fn create_texture_atlas(&mut self, images: &[&Handle<Image>]) -> Handle<TextureAtlas> {
        let mut hasher = ahash::AHasher::default();
        images.hash(&mut hasher);
        let id: u64 = hasher.finish();

        let handle = Handle::weak(HandleId::new_from_u64::<TextureAtlas>(id));

        self.set(
            handle,
            TextureAtlas {
                width: 0,
                height: 0,
                texture: None,
                image_handles: FxHashMap::default(),
                image_rects: Vec::with_capacity(0),
                images: images.iter().map(|i| ((*i).clone(), false)).collect(),
            },
        )
    }
}

pub(crate) fn prepare_texture_atlas_asset(
    texture_bind_group_layout: Option<Res<TextureBindGroupLayout>>,
    device: Option<Res<Device>>,
    queue: Option<Res<Queue>>,
    textures_atlas: Option<ResMut<Assets<TextureAtlas>>>,
    textures: Option<ResMut<Assets<Texture>>>,
    images: Option<ResMut<Assets<Image>>>,
    images_asset_event: EventStream<AssetEvent<Image>>,
) {
    let events = images_asset_event.read();
    if let (
        Some(mut textures_atlas),
        Some(mut textures),
        Some(mut images),
        Some(device),
        Some(queue),
        Some(texture_bind_group_layout),
    ) = (
        textures_atlas,
        textures,
        images,
        device,
        queue,
        texture_bind_group_layout,
    ) {
        for e in events {
            if let AssetEvent::Loaded(handle) = e {
                let image_handle = Handle::weak(handle.get_id());
                for (_, atlas) in textures_atlas
                    .iter_mut()
                    .filter(|(_, atlas)| !atlas.finalized())
                {
                    atlas.set_image_loaded(&image_handle);

                    if atlas.all_images_loaded() {
                        atlas.finalize_atlas(
                            &mut images,
                            &mut textures,
                            &device,
                            &queue,
                            &texture_bind_group_layout,
                        );
                    }
                }
            }
        }
    }
}

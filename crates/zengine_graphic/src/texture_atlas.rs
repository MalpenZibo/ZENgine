use std::collections::BTreeMap;

use glam::Vec2;
use rectangle_pack::{
    contains_smallest_box, pack_rects, volume_heuristic, GroupedRectsToPlace, PackedLocation,
    RectToInsert, RectanglePackError, TargetBin,
};
use rustc_hash::FxHashMap;
use zengine_asset::{Assets, Handle};
use zengine_macro::Asset;

use crate::Image;

const BIT_PER_PIXEL: usize = 8;

#[derive(Asset, Debug)]
pub struct TextureAtlas {
    image: Handle<Image>,
    image_rects: Vec<(Vec2, Vec2)>,
    image_handles: FxHashMap<Handle<Image>, usize>,
}

pub struct TextureAtlasBuilder {
    images: GroupedRectsToPlace<Handle<Image>>,
}

impl Default for TextureAtlasBuilder {
    fn default() -> Self {
        Self {
            images: GroupedRectsToPlace::new(),
        }
    }
}

impl TextureAtlasBuilder {
    pub fn add_image(mut self, image_handle: &Handle<Image>, image: &Image) -> Self {
        self.images.push_rect(
            image_handle.clone_as_weak(),
            None,
            RectToInsert::new(image.width, image.height, 1),
        );
        self
    }

    pub fn build(self, images: &mut Assets<Image>) -> TextureAtlas {
        let mut width = 256;
        let mut height = 256;

        let mut placements = None;
        let mut final_image = Image::default();

        while placements.is_none() {
            let mut containers = BTreeMap::new();
            containers.insert(0, TargetBin::new(width, height, 1));

            placements = match pack_rects(
                &self.images,
                &mut containers,
                &volume_heuristic,
                &contains_smallest_box,
            ) {
                Ok(placements) => {
                    final_image = Image::new(
                        width,
                        height,
                        vec![0; BIT_PER_PIXEL * (width * height) as usize],
                    );

                    Some(placements)
                }
                Err(RectanglePackError::NotEnoughBinSpace) => {
                    width *= 2;
                    height *= 2;
                    None
                }
            }
        }

        let placements = placements.unwrap();
        let mut image_handles = FxHashMap::default();
        let mut image_rects = Vec::with_capacity(placements.packed_locations().len());
        for (image_handle, (_, location)) in placements.packed_locations().iter() {
            let image = images.get(image_handle).unwrap();
            let min = Vec2::new(location.x() as f32, location.y() as f32);
            let max = min + Vec2::new(location.width() as f32, location.height() as f32);

            image_handles.insert(image_handle.clone_as_weak(), image_rects.len());
            image_rects.push((min, max));

            Self::copy_image_to_atlas(&mut final_image, image, location);
        }

        TextureAtlas {
            image: images.add(final_image),
            image_handles,
            image_rects,
        }
    }

    fn copy_image_to_atlas(atlas_image: &mut Image, image: &Image, location: &PackedLocation) {
        let source_width = location.width() as usize;
        let source_height = location.height() as usize;

        let target_width = atlas_image.width as usize;

        let x = location.x() as usize;
        let y = location.y() as usize;

        let target_data = atlas_image.data.as_mut().unwrap();
        let source_data = image.data.as_ref().unwrap();

        for (source_row, target_row) in (y..y + source_height).enumerate() {
            let target_begin = (target_row * target_width + x) * BIT_PER_PIXEL;
            let target_end = target_begin + source_width * BIT_PER_PIXEL;

            let source_begin = source_row * source_width * BIT_PER_PIXEL;
            let source_end = source_begin + source_width * BIT_PER_PIXEL;

            target_data[target_begin..target_end]
                .copy_from_slice(&source_data[source_begin..source_end])
        }
    }
}

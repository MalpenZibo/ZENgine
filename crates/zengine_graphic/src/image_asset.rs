use std::num::NonZeroU32;

use crate::{Device, Queue, TextureBindGroupLayout};
use image::{DynamicImage, GenericImageView};
use zengine_asset::{AssetEvent, AssetLoader, Assets};
use zengine_ecs::system::{EventStream, OptionalRes, OptionalResMut};
use zengine_macro::Asset;

#[derive(Asset, Default, Debug)]
pub struct Image {
    pub width: u32,
    pub height: u32,

    pub data: Vec<u8>,
}

impl Image {
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            data,
            ..Default::default()
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
            data: img.into_raw(),
        })
    }
}

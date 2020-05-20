extern crate image;

use image::{DynamicImage, GenericImageView};

pub struct ImageAsset {
    pub width: u32,
    pub height: u32,

    pub data: Vec<u8>,
}

pub fn load(image_name: &str) -> ImageAsset {
    match std::env::current_exe() {
        Ok(mut absolute_path) => {
            absolute_path.pop();

            absolute_path.push("assets/images/");
            absolute_path.push(image_name);

            match image::open(absolute_path) {
                Ok(img) => {
                    let (width, height) = img.dimensions();

                    let img = match img {
                        DynamicImage::ImageRgba8(img) => img,
                        img => img.to_rgba(),
                    };

                    return ImageAsset {
                        width: width,
                        height: height,
                        data: img.into_raw(),
                    };
                }
                Err(e) => panic!("Could not load image {}: {}", image_name, e),
            }
        }
        Err(e) => panic!("current exe path error: {}", e),
    }
}

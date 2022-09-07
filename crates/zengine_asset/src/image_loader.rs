extern crate image;

use image::{DynamicImage, GenericImageView};
use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug)]
pub struct ImageAsset {
    pub width: u32,
    pub height: u32,

    pub data: Vec<u8>,
}

unsafe impl Send for ImageAsset {}
unsafe impl Sync for ImageAsset {}

pub type ImageAssetHandler = Receiver<ImageAsset>;

pub fn load(image_name: &str) -> ImageAssetHandler {
    let image_name = image_name.to_owned();
    let (tx, rx) = std::sync::mpsc::channel::<ImageAsset>();

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {

            async fn load_image_background(image_name: String, sender: Sender<ImageAsset>) {
                use wasm_bindgen::JsCast;
                use js_sys::Uint8Array;
                use std::path::Path;

                let path = Path::new("assets/images/");
                let path = path.join(&image_name);
                let window = web_sys::window().unwrap();


                let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(path.to_str().unwrap())).await.unwrap();
                let resp: web_sys::Response = resp_value.dyn_into().unwrap();
                let resp_value = wasm_bindgen_futures::JsFuture::from(resp.array_buffer().unwrap()).await.unwrap();
                let bytes = Uint8Array::new(&resp_value).to_vec();

                let img = image::load_from_memory(&bytes).unwrap_or_else(|e| panic!("Could not load image {}: {}", image_name, e));

                let (width, height) = img.dimensions();

                let img = match img {
                    DynamicImage::ImageRgba8(img) => img,
                    img => img.to_rgba8(),
                };

                sender.send(ImageAsset {
                    width,
                    height,
                    data: img.into_raw(),
                }).expect("Error sending image asset");
            }

            wasm_bindgen_futures::spawn_local(load_image_background(image_name, tx));

        } else {
            fn load_image_background(image_name: String, sender: Sender<ImageAsset>) {
                let mut absolute_path = std::env::current_exe().unwrap_or_else(|e| panic!("current exe path error: {}", e));

                absolute_path.pop();

                absolute_path.push("assets/images/");
                absolute_path.push(&image_name);

                let img = image::open(absolute_path).unwrap_or_else(|e| panic!("Could not load image {}: {}", image_name, e));
                let (width, height) = img.dimensions();

                let img = match img {
                    DynamicImage::ImageRgba8(img) => img,
                    img => img.to_rgba8(),
                };

                sender.send(ImageAsset {
                    width,
                    height,
                    data: img.into_raw(),
                }).expect("Error sending image asset");

            }
            std::thread::spawn(move || load_image_background(image_name.to_owned(), tx));
        }
    }

    rx
}

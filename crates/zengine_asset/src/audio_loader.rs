extern crate image;

use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug)]
pub struct AudioAsset {
    pub data: Vec<u8>,
}

unsafe impl Send for AudioAsset {}
unsafe impl Sync for AudioAsset {}

pub type AudioAssetHandler = Receiver<AudioAsset>;

pub fn load(audio_name: &str) -> AudioAssetHandler {
    let audio_name = audio_name.to_owned();
    let (tx, rx) = std::sync::mpsc::channel::<AudioAsset>();

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {

            async fn load_audio_background(audio_name: String, sender: Sender<AudioAsset>) {
                use wasm_bindgen::JsCast;
                use js_sys::Uint8Array;
                use std::path::Path;

                let path = Path::new("assets/audio/");
                let path = path.join(&audio_name);
                let window = web_sys::window().unwrap();


                let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(path.to_str().unwrap())).await.unwrap();
                let resp: web_sys::Response = resp_value.dyn_into().unwrap();
                let resp_value = wasm_bindgen_futures::JsFuture::from(resp.array_buffer().unwrap()).await.unwrap();
                let bytes = Uint8Array::new(&resp_value).to_vec();

                sender.send(AudioAsset {
                    data: bytes.into(),
                }).expect("Error sending audio asset");
            }

            wasm_bindgen_futures::spawn_local(load_audio_background(audio_name, tx));

        } else {
            fn load_audio_background(audio_name: String, sender: Sender<AudioAsset>) {
                let mut absolute_path = std::env::current_exe().unwrap_or_else(|e| panic!("current exe path error: {}", e));

                absolute_path.pop();

                absolute_path.push("assets/audio/");
                absolute_path.push(&audio_name);

                let audio = std::fs::read(absolute_path).unwrap_or_else(|e| panic!("Could not load audio {}: {}", audio_name, e));

                sender.send(AudioAsset {
                    data: audio
                }).expect("Error sending image asset");
            }
            std::thread::spawn(move || load_audio_background(audio_name.to_owned(), tx));
        }
    }

    rx
}

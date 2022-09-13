use std::{path::Path};
use wasm_bindgen::JsCast;
use js_sys::Uint8Array;

pub async fn load(asset_path: &Path) -> Vec<u8> {
    let window = web_sys::window().unwrap();

    let resp_value = wasm_bindgen_futures::JsFuture::from(
        window.fetch_with_str(asset_path.to_str().unwrap()),
    )
    .await
    .unwrap();
    let resp: web_sys::Response = resp_value.dyn_into().unwrap();
    let resp_value = wasm_bindgen_futures::JsFuture::from(resp.array_buffer().unwrap())
        .await
        .unwrap();
    let data = Uint8Array::new(&resp_value).to_vec();

    data
}

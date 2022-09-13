use std::future::Future;

pub(crate) fn spawn<F: Future<Output = ()> + Send + 'static>(future: F) {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            wasm_bindgen_futures::spawn_local(future);
        } else {
            std::thread::spawn(|| pollster::block_on(future));
        }
    }
}

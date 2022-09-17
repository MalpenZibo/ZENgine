use std::future::Future;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        pub(crate) fn spawn<F: Future<Output = ()> + 'static>(future: F) {
            wasm_bindgen_futures::spawn_local(future);
        }
    } else {
        pub(crate) fn spawn<F: Future<Output = ()> + Send + 'static>(future: F) {
            std::thread::spawn(|| pollster::block_on(future));
        }
    }
}

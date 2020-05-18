extern crate zengine;

use zengine::core::Store;

fn main() {
    /*zengine::engine::start(
        zengine::engine::EngineOption {
            title: String::from("PONG"),
            fullscreen: false,
            virtual_width: 1920,
            virtual_height: 1080,
            screen_width: 800,
            screen_height: 600
        }
    );*/

    let mut store = Store::new();

    let entity = store.create_entity();

    println!("entity: {:?}", entity);
    println!("store: {:?}", store);
}

extern crate zengine;

use zengine::core::Scene;

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

    let mut scene = Scene::new();

    let entity = scene.create_entity();

    println!("entity: {:?}", entity);
    println!("scene: {:?}", scene);
}

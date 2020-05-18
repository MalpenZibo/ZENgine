extern crate zengine;

use zengine::core::Scene;
use zengine::core::Store;
use zengine::EngineBuilder;

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

    let engine = EngineBuilder::default().build();

    engine.run(Game)
}

pub struct Game;

impl Scene for Game {
    fn on_start(&mut self, store: &mut Store) {
        println!("Game scene on start");
    }

    fn on_stop(&mut self, store: &mut Store) {
        println!("Game scene on stop");
    }
}

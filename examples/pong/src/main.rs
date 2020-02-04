extern crate zengine;

fn main() {

    zengine::engine::start(
        zengine::engine::EngineOption {
            title: String::from("PONG"),
            fullscreen: false,
            virtual_width: 800,
            virtual_height: 600,
            screen_width: 1920,
            screen_height: 1080
        }
    );
}
use zengine::{log::Level, Engine};

#[cfg(not(target_os = "android"))]
fn main() {
    Engine::init_logger(Level::Info);

    Engine::default().add_system(hello_world_system).run();
}

fn hello_world_system() {
    println!("hello world");
}

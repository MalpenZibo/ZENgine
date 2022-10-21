use zengine::Engine;

#[cfg(not(target_os = "android"))]
fn main() {
    Engine::default().add_system(hello_world_system).run();
}

fn hello_world_system() {
    println!("hello world");
}

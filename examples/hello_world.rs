use zengine::Engine;

fn main() {
    Engine::default().add_system(hello_world_system).run();
}

fn hello_world_system() {
    println!("hello world");
}

use crate::core::store::Store;

pub enum Trans {
    None,
    /// Stop and shut down the engine.
    Quit,
}

pub trait Scene {
    #[allow(unused_variables)]
    fn on_start(&mut self, store: &mut Store) {}

    #[allow(unused_variables)]
    fn on_stop(&mut self, store: &mut Store) {}

    fn update(&mut self, store: &mut Store) -> Trans {
        Trans::None
    }
}

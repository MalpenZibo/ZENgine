use crate::core::event::EventStream;
use crate::core::store::Store;

#[derive(Clone)]
pub enum Trans {
    None,
    /// Stop and shut down the engine.
    Quit,
}

pub trait AnyScene {
    #[allow(unused_variables)]
    fn on_start(&mut self, store: &mut Store);

    #[allow(unused_variables)]
    fn on_stop(&mut self, store: &mut Store);

    #[allow(unused_variables)]
    fn update_wrapper(&mut self, store: &Store) -> Trans;
}

impl<S> AnyScene for S
where
    S: Scene,
{
    #[allow(unused_variables)]
    fn on_start(&mut self, store: &mut Store) {
        self.on_start(store);
    }

    #[allow(unused_variables)]
    fn on_stop(&mut self, store: &mut Store) {
        self.on_stop(store);
    }

    fn update_wrapper(&mut self, store: &Store) -> Trans {
        let mut received_trans = Trans::None;
        if let Some(stream) = store.get_resource::<EventStream<Trans>>() {
            if let Some(trans) = stream.read_last() {
                received_trans = match trans {
                    Trans::Quit => Trans::Quit,
                    _ => Trans::None,
                }
            }
        }
        match received_trans {
            Trans::Quit => Trans::Quit,
            _ => self.update(store),
        }
    }
}

pub trait Scene {
    #[allow(unused_variables)]
    fn on_start(&mut self, store: &mut Store) {}

    #[allow(unused_variables)]
    fn on_stop(&mut self, store: &mut Store) {}

    fn update(&mut self, store: &Store) -> Trans {
        Trans::None
    }
}

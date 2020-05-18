use crate::core::store::Store;

pub trait Scene {
  fn on_start(&mut self, store: &mut Store) {}

  fn on_stop(&mut self, store: &mut Store) {}
}

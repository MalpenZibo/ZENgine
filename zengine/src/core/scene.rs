use crate::core::store::Store;

pub trait Scene {
  #[allow(unused_variables)]
  fn on_start(&mut self, store: &mut Store) {}

  #[allow(unused_variables)]
  fn on_stop(&mut self, store: &mut Store) {}
}

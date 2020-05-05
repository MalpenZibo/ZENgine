use crate::core::entity::Entity;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Scene {
  entity_cur_id: u32,
  pub entities: HashMap<u32, Entity>,
}

impl Scene {
  pub fn new() -> Self {
    Scene {
      entity_cur_id: 0,
      entities: HashMap::new(),
    }
  }

  pub fn create_entity(&mut self) -> &Entity {
    let new_id = self.entity_cur_id;
    self.entity_cur_id += 1;

    self.entities.insert(new_id, Entity { id: new_id });
    self.entities.get(&new_id).unwrap()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn create_entity() {
    let mut scene = Scene::new();
    let entity = scene.create_entity();

    assert_eq!(entity.id, 0);
    assert_eq!(scene.entity_cur_id, 1);
  }
}

use crate::core::entity::Entity;
use crate::core::entity::EntityId;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Scene {
  entity_cur_id: u32,
  pub entities: HashMap<EntityId, Entity>,
}

impl Scene {
  pub fn new() -> Self {
    Scene {
      entity_cur_id: 0,
      entities: HashMap::new(),
    }
  }

  pub fn create_entity(&mut self) -> EntityId {
    let new_id = EntityId(self.entity_cur_id);
    self.entity_cur_id += 1;

    self.entities.insert(new_id, Entity { id: new_id });
    new_id
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn create_entity() {
    let mut scene = Scene::new();
    let entity_id = scene.create_entity();

    assert_eq!(entity_id, EntityId(0));
    assert_eq!(scene.entity_cur_id, 1);
  }
}

use crate::core::component_manager::ComponentManager;
use crate::core::entity::{Entity, EntityId};
use std::any::Any;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Scene {
  entity_cur_id: u32,
  pub entities: HashMap<EntityId, Entity>,
  pub component_manager: ComponentManager,
}

impl Scene {
  pub fn new() -> Self {
    Scene {
      entity_cur_id: 0,
      entities: HashMap::new(),
      component_manager: ComponentManager::new(),
    }
  }

  pub fn get_entity(&self, entity_id: &EntityId) -> Option<&Entity> {
    self.entities.get(entity_id)
  }

  pub fn create_entity(&mut self) -> EntityId {
    let new_id = EntityId(self.entity_cur_id);
    self.entity_cur_id += 1;

    self.entities.insert(new_id, Entity { id: new_id });
    new_id
  }

  pub fn get_component<C: Any>(&self, entity_id: &EntityId) -> Option<&C> {
    self.component_manager.get(entity_id)
  }

  pub fn add_component<C: Any>(&mut self, entity_id: &EntityId, component: C) {
    self.component_manager.add(entity_id, component);
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

  #[test]
  fn retrieve_entity_after_creation() {
    let mut scene = Scene::new();
    let entity_id = scene.create_entity();

    let entity = scene.get_entity(&entity_id);

    assert_ne!(entity, None);
    assert_eq!(entity, Some(&Entity { id: EntityId(0) }));
  }

  #[test]
  fn retrieve_entity_wrong_id() {
    let mut scene = Scene::new();
    scene.create_entity();

    let entity = scene.get_entity(&EntityId(8));

    assert_eq!(entity, None);
  }

  #[test]
  fn add_and_retrieve_component() {
    #[derive(PartialEq, Debug)]
    struct Test {
      data1: i32,
      data2: f32,
    };
    let mut scene = Scene::new();

    let entity_id = scene.create_entity();
    scene.add_component(
      &entity_id,
      Test {
        data1: 4,
        data2: 8.5,
      },
    );

    let test: Option<&Test> = scene.get_component(&entity_id);
    assert_eq!(
      test,
      Some(&Test {
        data1: 4,
        data2: 8.5
      })
    );
  }

  #[test]
  fn retrieve_component_not_present() {
    #[derive(PartialEq, Debug)]
    struct Test {
      data1: i32,
      data2: f32,
    };
    let mut scene = Scene::new();

    let entity_id = scene.create_entity();

    let test: Option<&Test> = scene.get_component(&entity_id);
    assert_eq!(test, None);
  }
}

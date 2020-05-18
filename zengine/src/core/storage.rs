use crate::core::entity::EntityId;
use std::collections::HashMap;

pub struct Storage<C> {
  data: HashMap<EntityId, C>,
}

impl<C> Storage<C> {
  pub fn new() -> Self {
    Self {
      data: HashMap::new(),
    }
  }

  pub fn insert(&mut self, entity: &EntityId, data: C) {
    self.data.insert(entity.clone(), data);
  }

  pub fn get(&self, entity: &EntityId) -> Option<&C> {
    self.data.get(entity)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(PartialEq, Debug)]
  struct Component1 {
    data1: i32,
    data2: f32,
  }

  #[test]
  fn get_from_empty_storage() {
    let storage: Storage<Component1> = Storage::new();
    let entity_id = EntityId(5);

    let component = storage.get(&entity_id);

    assert_eq!(component, None);
  }

  #[test]
  fn insert_and_get_from_storage() {
    let mut storage: Storage<Component1> = Storage::new();
    let entity_id = EntityId(5);

    storage.insert(
      &entity_id,
      Component1 {
        data1: 3,
        data2: 3.5,
      },
    );
    let component = storage.get(&entity_id);

    assert_eq!(storage.data.len(), 1);
    assert_eq!(
      component,
      Some(&Component1 {
        data1: 3,
        data2: 3.5,
      })
    );
  }
}

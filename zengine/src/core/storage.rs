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

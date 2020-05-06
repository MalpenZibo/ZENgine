use crate::core::entity::EntityId;
use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ComponentManager {
  stores: HashMap<TypeId, Box<dyn Any>>,
}

impl ComponentManager {
  pub fn new() -> Self {
    ComponentManager {
      stores: HashMap::new(),
    }
  }

  pub fn get<C: Any>(&self, entity_id: &EntityId) -> Option<&C> {
    let type_id = TypeId::of::<C>();
    match self.stores.get(&type_id) {
      Some(generic_store) => {
        let target_store = generic_store
          .downcast_ref::<HashMap<EntityId, C>>()
          .unwrap();

        target_store.get(&entity_id)
      }
      None => None,
    }
  }

  pub fn add<C: Any>(&mut self, entity_id: &EntityId, component: C) {
    let type_id = TypeId::of::<C>();
    if let Some(generic_store) = self.stores.get_mut(&type_id) {
      let target_store = generic_store
        .downcast_mut::<HashMap<EntityId, C>>()
        .unwrap();
      target_store.insert(entity_id.clone(), component);
    } else {
      let mut store: HashMap<EntityId, C> = HashMap::new();
      store.insert(entity_id.clone(), component);
      self.stores.insert(type_id, Box::new(store));
    }
  }
}

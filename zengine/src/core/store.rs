use crate::core::component_manager::ComponentManager;
use crate::core::entity::{Entity, EntityId};
use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Store {
  entity_cur_id: u32,
  pub resources: HashMap<TypeId, Box<dyn Any>>,
  pub entities: HashMap<EntityId, Entity>,
  pub component_manager: ComponentManager,
}

impl Store {
  pub fn new() -> Self {
    Store {
      entity_cur_id: 0,
      resources: HashMap::new(),
      entities: HashMap::new(),
      component_manager: ComponentManager::new(),
    }
  }

  pub fn get<R: Any>(&self) -> Option<&R> {
    let type_id = TypeId::of::<R>();

    match self.resources.get(&type_id) {
      Some(res) => res.downcast_ref::<R>(),
      None => None,
    }
  }

  pub fn get_mut<R: Any>(&mut self) -> Option<&mut R> {
    let type_id = TypeId::of::<R>();

    match self.resources.get_mut(&type_id) {
      Some(res) => res.downcast_mut::<R>(),
      None => None,
    }
  }

  pub fn insert<R: Any>(&mut self, res: R) {
    let type_id = TypeId::of::<R>();
    self.resources.insert(type_id, Box::new(res));
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
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::core::storage::Storage;

  struct Resource {
    possible_data: i32,
  }

  impl Resource {
    pub fn double_data(&self) -> i32 {
      self.possible_data * 2
    }

    pub fn change_data(&mut self, new_data: i32) -> i32 {
      self.possible_data = new_data;
      self.possible_data
    }
  }

  #[test]
  fn insert_resource() {
    let mut store = Store::new();

    store.insert(Resource { possible_data: 3 });

    assert_eq!(store.resources.len(), 1);
  }

  #[test]
  fn insert_and_get_immutable_resource() {
    let mut store = Store::new();

    store.insert(Resource { possible_data: 3 });

    let immut_res: &Resource = store.get().unwrap();

    assert_eq!(immut_res.double_data(), 6);
  }

  #[test]
  fn insert_and_get_mutable_resource() {
    let mut store = Store::new();

    store.insert(Resource { possible_data: 3 });

    let mut_res: &mut Resource = store.get_mut().unwrap();

    assert_eq!(mut_res.change_data(8), 8);
  }

  #[derive(PartialEq, Debug)]
  struct Component1 {
    data1: i32,
    data2: f32,
  }

  struct Setup {
    store: Store,
    entity_id: EntityId,
  }

  fn create_store_with_entity() -> Setup {
    let mut store = Store::new();
    let entity_id = store.create_entity();

    Setup {
      store: store,
      entity_id: entity_id,
    }
  }

  #[test]
  fn create_entity() {
    let setup = create_store_with_entity();

    assert_eq!(setup.entity_id, EntityId(0));
    assert_eq!(setup.store.entity_cur_id, 1);
  }

  #[test]
  fn retrieve_entity_after_creation() {
    let setup = create_store_with_entity();

    let entity = setup.store.get_entity(&setup.entity_id);

    assert_ne!(entity, None);
    assert_eq!(entity, Some(&Entity { id: EntityId(0) }));
  }

  #[test]
  fn retrieve_entity_wrong_id() {
    let mut store = Store::new();
    store.create_entity();

    let entity = store.get_entity(&EntityId(8));

    assert_eq!(entity, None);
  }

  #[test]
  fn retrieve_component_not_present() {
    let mut setup = create_store_with_entity();

    let storage: Storage<Component1> = Storage::new();
    setup.store.insert(storage);

    let get_ref = setup.store.get::<Storage<Component1>>();
    if let Some(storage) = get_ref {
      let component = storage.get(&setup.entity_id);
      assert_eq!(component, None);
    }
  }

  #[test]
  fn add_and_retrieve_component() {
    let mut setup = create_store_with_entity();

    let storage: Storage<Component1> = Storage::new();
    setup.store.insert(storage);

    let get_ref = setup.store.get_mut::<Storage<Component1>>();
    if let Some(mut_storage) = get_ref {
      mut_storage.insert(
        &setup.entity_id,
        Component1 {
          data1: 5,
          data2: 2.3,
        },
      );
      let component = mut_storage.get(&setup.entity_id);
      assert_eq!(
        component,
        Some(&Component1 {
          data1: 5,
          data2: 2.3
        })
      );
    }
  }
}

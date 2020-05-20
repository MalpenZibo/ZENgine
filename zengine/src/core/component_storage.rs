use crate::core::entity::Entity;
use crate::core::store::Resource;
use downcast_rs::Downcast;
use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt::Debug;

pub trait Component: Any + Debug {}

#[derive(Debug)]
pub struct ComponentStorageResource {
  storages: HashMap<TypeId, Box<dyn AnyStorage>>,
}

impl Resource for ComponentStorageResource {}

impl Default for ComponentStorageResource {
  fn default() -> Self {
    ComponentStorageResource {
      storages: HashMap::new(),
    }
  }
}

impl ComponentStorageResource {
  pub fn get<C: Component>(&self) -> Option<&ComponentStorage<C>> {
    let type_id = TypeId::of::<C>();

    match self.storages.get(&type_id) {
      Some(storage) => storage.downcast_ref::<ComponentStorage<C>>(),
      None => None,
    }
  }

  pub fn add_component<C: Component>(&mut self, entity: &Entity, component: C) {
    let type_id = TypeId::of::<C>();

    match self.storages.get_mut(&type_id) {
      Some(storage) => storage
        .downcast_mut::<ComponentStorage<C>>()
        .expect("downcast storage error")
        .insert(entity, component),
      None => {
        let mut storage = ComponentStorage::<C>::default();
        storage.insert(entity, component);

        self.storages.insert(type_id, Box::new(storage));
      }
    }
  }

  pub fn delete_entity(&mut self, entity: &Entity) {
    for s in self.storages.iter_mut() {
      s.1.remove(entity);
    }
  }

  pub fn delete_all(&mut self) {
    for s in self.storages.iter_mut() {
      s.1.clear();
    }
  }
}

pub trait AnyStorage: Downcast + Debug {
  fn remove(&mut self, entity: &Entity);

  fn clear(&mut self);
}
downcast_rs::impl_downcast!(AnyStorage);

#[derive(Debug)]
pub struct ComponentStorage<C> {
  data: HashMap<Entity, C>,
}

impl<C> Default for ComponentStorage<C> {
  fn default() -> Self {
    ComponentStorage {
      data: HashMap::new(),
    }
  }
}

impl<C: Component> AnyStorage for ComponentStorage<C> {
  fn remove(&mut self, entity: &Entity) {
    self.data.remove(&entity);
  }

  fn clear(&mut self) {
    self.data.clear();
  }
}

impl<C> ComponentStorage<C> {
  pub fn insert(&mut self, entity: &Entity, data: C) {
    self.data.insert(entity.clone(), data);
  }

  pub fn get(&self, entity: &Entity) -> Option<&C> {
    self.data.get(entity)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::core::store::Store;

  #[derive(PartialEq, Debug)]
  struct Component1 {
    data1: i32,
    data2: f32,
  }

  impl Component for Component1 {}

  #[test]
  fn get_from_empty_storage() {
    let mut store = Store::default();
    let entity = store.build_entity().build();

    let storage: ComponentStorage<Component1> = ComponentStorage::default();
    let component = storage.get(&entity);

    assert_eq!(component, None);
  }

  #[test]
  fn insert_and_get_from_storage() {
    let mut store = Store::default();
    let entity = store.build_entity().build();
    let mut storage: ComponentStorage<Component1> = ComponentStorage::default();

    storage.insert(
      &entity,
      Component1 {
        data1: 3,
        data2: 3.5,
      },
    );
    let component = storage.get(&entity);

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

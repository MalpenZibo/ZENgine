use crate::core::entity::Entity;
use crate::core::store::Resource;
use downcast_rs::Downcast;
use std::any::Any;
use std::any::TypeId;
use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::HashMap;
use std::fmt::Debug;

pub trait Component: Any + Debug {}

#[derive(Debug)]
pub struct Components {
    storages: HashMap<TypeId, RefCell<Box<dyn AnySet>>>,
}

impl Resource for Components {}

impl Default for Components {
    fn default() -> Self {
        Components {
            storages: HashMap::new(),
        }
    }
}

impl Components {
    pub fn get<C: Component>(&self) -> Option<Ref<Set<C>>> {
        let type_id = TypeId::of::<C>();

        match self.storages.get(&type_id) {
            Some(storage) => Some(Ref::map(storage.borrow(), |b| {
                b.downcast_ref::<Set<C>>().expect("downcast set error")
            })),
            None => None,
        }
    }

    pub fn get_mut<C: Component>(&self) -> Option<RefMut<Set<C>>> {
        let type_id = TypeId::of::<C>();

        match self.storages.get(&type_id) {
            Some(storage) => Some(RefMut::map(storage.borrow_mut(), |b| {
                b.downcast_mut::<Set<C>>().expect("downcast set error")
            })),
            None => None,
        }
    }

    pub fn add_component<C: Component>(&mut self, entity: &Entity, component: C) {
        let type_id = TypeId::of::<C>();

        match self.storages.get_mut(&type_id) {
            Some(storage) => {
                RefMut::map(storage.borrow_mut(), |b| {
                    b.downcast_mut::<Set<C>>().expect("downcast set error")
                })
                .insert(entity.clone(), component);
            }
            None => {
                let mut storage = Set::<C>::default();
                storage.insert(entity.clone(), component);

                self.storages
                    .insert(type_id, RefCell::new(Box::new(storage)));
            }
        }
    }

    pub fn delete_entity(&mut self, entity: &Entity) {
        for s in self.storages.iter() {
            s.1.borrow_mut().remove(entity);
        }
    }

    pub fn delete_all(&mut self) {
        for s in self.storages.iter_mut() {
            s.1.borrow_mut().clear();
        }
    }
}

pub trait AnySet: Downcast + Debug {
    fn remove(&mut self, entity: &Entity);

    fn clear(&mut self);
}
downcast_rs::impl_downcast!(AnySet);

pub type Set<C> = HashMap<Entity, C>;
impl<C: Component> AnySet for Set<C> {
    fn remove(&mut self, entity: &Entity) {
        self.remove(&entity);
    }

    fn clear(&mut self) {
        self.clear();
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

        let storage: Set<Component1> = Set::default();
        let component = storage.get(&entity);

        assert_eq!(component, None);
    }

    #[test]
    fn insert_and_get_from_storage() {
        let mut store = Store::default();
        let entity = store.build_entity().build();
        let mut storage: Set<Component1> = Set::default();

        storage.insert(
            entity,
            Component1 {
                data1: 3,
                data2: 3.5,
            },
        );
        let component = storage.get(&entity);

        assert_eq!(storage.len(), 1);
        assert_eq!(
            component,
            Some(&Component1 {
                data1: 3,
                data2: 3.5,
            })
        );
    }
}

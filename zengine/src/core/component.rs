use crate::core::entity::Entity;
use crate::core::store::Resource;
use downcast_rs::Downcast;
use fnv::FnvHashMap;
use std::any::Any;
use std::any::TypeId;
use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::hash_map::Iter;
use std::collections::hash_map::IterMut;
use std::fmt::Debug;

pub trait Component: Any + Debug {}

#[derive(Resource, Debug)]
pub struct Components {
    storages: FnvHashMap<TypeId, RefCell<Box<dyn AnySet>>>,
}

impl Default for Components {
    fn default() -> Self {
        Components {
            storages: FnvHashMap::default(),
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

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn insert_component<C: Component>(&mut self, entity: &Entity, component: C) {
        let type_id = TypeId::of::<C>();

        match self.storages.get_mut(&type_id) {
            Some(storage) => {
                RefMut::map(storage.borrow_mut(), |b| {
                    b.downcast_mut::<Set<C>>().expect("downcast set error")
                })
                .insert(*entity, component);
            }
            None => {
                let mut storage = Set::<C>::default();
                storage.insert(*entity, component);

                self.storages
                    .insert(type_id, RefCell::new(Box::new(storage)));
            }
        }
    }

    pub fn register_component<C: Component>(&mut self) {
        let type_id = TypeId::of::<C>();

        if self.storages.get(&type_id).is_none() {
            self.storages
                .insert(type_id, RefCell::new(Box::new(Set::<C>::default())));
        }
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn remove_entity(&mut self, entity: &Entity) {
        for s in self.storages.iter() {
            s.1.borrow_mut().remove(entity);
        }
    }

    pub fn clear(&mut self) {
        for s in self.storages.iter_mut() {
            s.1.borrow_mut().clear();
        }
    }
}

pub trait AnySet: Downcast + Debug {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn remove(&mut self, entity: &Entity);

    fn clear(&mut self);
}
downcast_rs::impl_downcast!(AnySet);

pub type Set<C> = FnvHashMap<Entity, C>;
impl<C: Component> AnySet for Set<C> {
    fn remove(&mut self, entity: &Entity) {
        self.remove(&entity);
    }

    fn clear(&mut self) {
        self.clear();
    }
}

pub struct Join<'a, C1: Component, C2: Component> {
    inner: Iter<'a, Entity, C1>,
    others: Iter<'a, Entity, C2>,
}

impl<'a, C1: Component, C2: Component> Iterator for Join<'a, C1, C2> {
    type Item = (&'a Entity, &'a C1, &'a C2);

    fn next(&mut self) -> Option<Self::Item> {
        let mut next_value: Option<Self::Item> = None;

        while let Some(entry) = self.inner.next() {
            if let Some(other) = self.others.find(|other_entry| other_entry.0 == entry.0) {
                next_value = Some((entry.0, entry.1, other.1));
                break;
            }
        }

        next_value
    }
}

pub struct JoinMut<'a, C1: Component, C2: Component> {
    inner: IterMut<'a, Entity, C1>,
    others: IterMut<'a, Entity, C2>,
}

impl<'a, C1: Component, C2: Component> Iterator for JoinMut<'a, C1, C2> {
    type Item = (&'a Entity, &'a mut C1, &'a mut C2);

    fn next(&mut self) -> Option<Self::Item> {
        let mut next_value: Option<Self::Item> = None;

        while let Some(entry) = self.inner.next() {
            if let Some(other) = self.others.find(|other_entry| other_entry.0 == entry.0) {
                next_value = Some((entry.0, entry.1, other.1));
                break;
            }
        }

        next_value
    }
}

pub trait Joinable<C1: Component> {
    fn join<'a, C2: Component>(&'a self, others: &'a Set<C2>) -> Join<C1, C2>;

    fn join_mut<'a, C2: Component>(&'a mut self, others: &'a mut Set<C2>) -> JoinMut<C1, C2>;
}

impl<C1: Component> Joinable<C1> for Set<C1> {
    fn join<'a, C2: Component>(&'a self, others: &'a Set<C2>) -> Join<C1, C2> {
        Join {
            inner: self.iter(),
            others: others.iter(),
        }
    }

    fn join_mut<'a, C2: Component>(&'a mut self, others: &'a mut Set<C2>) -> JoinMut<C1, C2> {
        JoinMut {
            inner: self.iter_mut(),
            others: others.iter_mut(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::store::Store;

    #[derive(Component, PartialEq, Debug)]
    struct Component1 {
        data1: i32,
        data2: f32,
    }

    #[derive(Component, PartialEq, Debug)]
    struct Component2 {
        data3: String,
        data4: u32,
    }

    #[derive(Component, PartialEq, Debug)]
    struct Component3 {
        data5: i32,
    }

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

    fn prapare_join_test() -> (Set<Component1>, Set<Component2>, Set<Component3>) {
        let mut store = Store::default();
        let entity1 = store.build_entity().build();
        let entity2 = store.build_entity().build();
        let entity3 = store.build_entity().build();
        let entity4 = store.build_entity().build();
        let mut storage1: Set<Component1> = Set::default();
        let mut storage2: Set<Component2> = Set::default();
        let mut storage3: Set<Component3> = Set::default();

        storage1.insert(
            entity1,
            Component1 {
                data1: 3,
                data2: 3.5,
            },
        );

        storage1.insert(
            entity2,
            Component1 {
                data1: 6,
                data2: 7.8,
            },
        );

        storage1.insert(
            entity3,
            Component1 {
                data1: 9,
                data2: 2.1,
            },
        );

        storage1.insert(
            entity4,
            Component1 {
                data1: 3,
                data2: 3.5,
            },
        );

        storage2.insert(
            entity1,
            Component2 {
                data3: "test".to_string(),
                data4: 2,
            },
        );

        storage2.insert(
            entity2,
            Component2 {
                data3: "test2".to_string(),
                data4: 7,
            },
        );

        storage3.insert(entity1, Component3 { data5: 5 });

        storage3.insert(entity2, Component3 { data5: 5 });

        storage3.insert(entity3, Component3 { data5: 5 });

        storage3.insert(entity4, Component3 { data5: 5 });

        (storage1, storage2, storage3)
    }

    #[test]
    fn join_iterator() {
        let (storage1, storage2, storage3) = prapare_join_test();

        assert_eq!(storage1.join(&storage2).count(), 2);

        for (entity, c1, c2) in storage1.join(&storage2) {
            println!("{:?}", c1.data1);
        }
    }

    #[test]
    fn join_iterator_mut() {
        let (mut storage1, mut storage2, mut storage3) = prapare_join_test();

        assert_eq!(storage1.join_mut(&mut storage2).count(), 2);

        for (entity, mut c1, mut c2) in storage1.join_mut(&mut storage2) {
            println!("{:?}", c1.data1);
            c1.data1 = 5;
            c2.data4 = 7;
        }
    }
}

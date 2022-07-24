use crate::core::component::Component;
use crate::core::component::Components;
use crate::core::component::Set;
use crate::core::entity::{Entities, Entity, EntityBuilder};
use crate::event::EventStream;
use crate::event::SubscriptionToken;
use downcast_rs::Downcast;
use fnv::FnvHashMap;
use std::any::Any;
use std::any::TypeId;
use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;

pub trait Resource: Downcast + 'static {}
downcast_rs::impl_downcast!(Resource);

#[derive(Default)]
pub struct Store {
    entities: Entities,
    components: Components,
    resources: FnvHashMap<TypeId, RefCell<Box<dyn Resource>>>,
}

impl Store {
    pub fn get_resource<R: Resource>(&self) -> Option<Ref<R>> {
        let type_id = TypeId::of::<R>();

        self.resources.get(&type_id).map(|resource| {
            Ref::map(resource.borrow(), |b| {
                b.downcast_ref::<R>().expect("downcast resource error")
            })
        })
    }

    pub fn get_resource_mut<R: Resource>(&self) -> Option<RefMut<R>> {
        let type_id = TypeId::of::<R>();

        self.resources.get(&type_id).map(|resource| {
            RefMut::map(resource.borrow_mut(), |b| {
                b.downcast_mut::<R>().expect("downcast resource error")
            })
        })
    }

    pub fn get_entities(&self) -> &Entities {
        &self.entities
    }

    pub fn insert_resource<R: Resource>(&mut self, res: R) {
        let type_id = TypeId::of::<R>();
        self.resources.insert(type_id, RefCell::new(Box::new(res)));
    }

    pub fn build_entity(&mut self) -> EntityBuilder {
        EntityBuilder {
            entity: self.entities.create_entity(),
            store: self,
            is_build: false,
        }
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn remove_entity(&mut self, entity: &Entity) {
        self.components.remove_entity(entity);
    }

    pub fn delete_all(&mut self) {
        self.components.clear();
    }

    pub fn get_components<C: Component>(&self) -> Option<Ref<Set<C>>> {
        self.components.get::<C>()
    }

    pub fn get_components_mut<C: Component>(&self) -> Option<RefMut<Set<C>>> {
        self.components.get_mut::<C>()
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn insert_component<C: Component>(&mut self, entity: &Entity, component: C) {
        self.components.insert_component(entity, component);
    }

    pub fn register_component<C: Component>(&mut self) {
        self.components.register_component::<C>();
    }

    pub fn subscribe<E: Any>(&mut self) -> SubscriptionToken {
        let mut stream = self
            .get_resource_mut::<EventStream<E>>()
            .unwrap_or_else(|| {
                panic!(
                    "No stream registered for event of type: {:?}",
                    TypeId::of::<E>()
                )
            });

        stream.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource, Debug, Default)]
    struct Resource1 {
        possible_data: i32,
    }

    impl Resource1 {
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
        let mut store = Store::default();

        store.insert_resource(Resource1 { possible_data: 3 });

        assert_eq!(store.resources.len(), 1);
    }

    #[test]
    fn insert_and_get_immutable_resource() {
        let mut store = Store::default();

        store.insert_resource(Resource1 { possible_data: 3 });

        let immut_res: Ref<Resource1> = store.get_resource().unwrap();

        assert_eq!(immut_res.double_data(), 6);
    }

    #[test]
    fn insert_and_get_mutable_resource() {
        let mut store = Store::default();

        store.insert_resource(Resource1 { possible_data: 3 });

        let mut mut_res: RefMut<Resource1> = store.get_resource_mut().unwrap();

        assert_eq!(mut_res.change_data(8), 8);
    }

    #[derive(Component, PartialEq, Debug)]
    struct Component1 {
        data1: i32,
        data2: f32,
    }

    #[test]
    fn create_entity_with_builder() {
        let mut store = Store::default();

        let entity = store
            .build_entity()
            .with(Component1 {
                data1: 2,
                data2: 6.5,
            })
            .build();

        let components = store.get_components::<Component1>().unwrap();
        let component = components.get(&entity).unwrap();
        assert_eq!(
            component,
            &Component1 {
                data1: 2,
                data2: 6.5,
            }
        );
    }

    #[test]
    fn create_entity_with_builder_without_build() {
        let mut store = Store::default();

        let entity;
        {
            let entity_builder = store.build_entity().with(Component1 {
                data1: 2,
                data2: 6.5,
            });

            entity = entity_builder.entity.clone();
        }

        let components = store.get_components::<Component1>().unwrap();
        let component = components.get(&entity);

        assert_eq!(component, None);
    }
}

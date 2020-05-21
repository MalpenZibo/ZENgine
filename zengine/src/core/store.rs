use crate::core::component_storage::Component;
use crate::core::component_storage::ComponentStorage;
use crate::core::component_storage::ComponentStorageResource;
use crate::core::entity::{EntitiesResource, Entity, EntityBuilder};
use downcast_rs::Downcast;
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt::Debug;

pub trait Resource: Downcast + Debug + 'static {}
downcast_rs::impl_downcast!(Resource);

#[derive(Debug)]
pub struct Store {
    entities: EntitiesResource,
    component_storage: ComponentStorageResource,
    resources: HashMap<TypeId, Box<dyn Resource>>,
}

impl Default for Store {
    fn default() -> Self {
        Store {
            entities: EntitiesResource::default(),
            component_storage: ComponentStorageResource::default(),
            resources: HashMap::new(),
        }
    }
}

impl Store {
    pub fn get_resource<R: Resource>(&self) -> Option<&R> {
        let type_id = TypeId::of::<R>();

        match self.resources.get(&type_id) {
            Some(res) => res.downcast_ref::<R>(),
            None => None,
        }
    }

    pub fn get_resource_mut<R: Resource>(&mut self) -> Option<&mut R> {
        let type_id = TypeId::of::<R>();

        match self.resources.get_mut(&type_id) {
            Some(res) => res.downcast_mut::<R>(),
            None => None,
        }
    }

    pub fn insert_resource<R: Resource>(&mut self, res: R) {
        let type_id = TypeId::of::<R>();
        self.resources.insert(type_id, Box::new(res));
    }

    pub fn build_entity(&mut self) -> EntityBuilder {
        EntityBuilder {
            entity: self.entities.create_entity(),
            store: self,
            is_build: false,
        }
    }

    pub fn delete_entity(&mut self, entity: &Entity) {
        self.component_storage.delete_entity(entity);
    }

    pub fn delete_all(&mut self) {
        self.component_storage.delete_all();
    }

    pub fn get_component_storage<C: Component>(&self) -> Option<&ComponentStorage<C>> {
        self.component_storage.get::<C>()
    }

    pub fn insert_component<C: Component>(&mut self, entity: &Entity, component: C) {
        self.component_storage.add_component(entity, component);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Resource1 {
        possible_data: i32,
    }

    impl Resource for Resource1 {}

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

        let immut_res: &Resource1 = store.get_resource().unwrap();

        assert_eq!(immut_res.double_data(), 6);
    }

    #[test]
    fn insert_and_get_mutable_resource() {
        let mut store = Store::default();

        store.insert_resource(Resource1 { possible_data: 3 });

        let mut_res: &mut Resource1 = store.get_resource_mut().unwrap();

        assert_eq!(mut_res.change_data(8), 8);
    }

    #[derive(PartialEq, Debug)]
    struct Component1 {
        data1: i32,
        data2: f32,
    }

    impl Component for Component1 {}

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

        let components = store.get_component_storage::<Component1>().unwrap();
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

        let components = store.get_component_storage::<Component1>().unwrap();
        let component = components.get(&entity);

        assert_eq!(component, None);
    }
}

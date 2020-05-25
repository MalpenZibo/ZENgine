use crate::core::component::Component;
use crate::core::store::Resource;
use crate::core::store::Store;
use std::fmt::Debug;

#[derive(Default, Debug)]
pub struct Entities {
    max_id: u32,
}

impl Resource for Entities {}

impl Entities {
    pub fn create_entity(&mut self) -> Entity {
        let id = self.max_id;
        self.max_id += 1;

        Entity(id)
    }
}

pub struct EntityBuilder<'a> {
    pub entity: Entity,
    pub store: &'a mut Store,
    pub is_build: bool,
}

impl<'a> EntityBuilder<'a> {
    pub fn with<C: Component>(self, component: C) -> Self {
        self.store.insert_component(&self.entity, component);

        self
    }

    pub fn build(mut self) -> Entity {
        self.is_build = true;

        self.entity
    }
}

impl<'a> Drop for EntityBuilder<'a> {
    fn drop(&mut self) {
        if !self.is_build {
            self.store.delete_entity(&self.entity);
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Entity(u32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_entity() {
        let mut er = Entities::default();

        let entity = er.create_entity();

        assert_eq!(entity, Entity(0));
    }
}

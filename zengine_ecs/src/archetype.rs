use crate::{
    component::{component_vec_to_mut, Component, ComponentColumn},
    entity::Entity,
};
use std::{
    any::TypeId,
    hash::{Hash, Hasher},
    sync::RwLock,
};

pub type ArchetypeId = u64;
pub type ArchetypeSpecs = Vec<TypeId>;

pub fn calculate_archetype_id(types: &[TypeId]) -> ArchetypeId {
    let mut s = rustc_hash::FxHasher::default();
    types.hash(&mut s);
    s.finish()
}

#[derive(Default, Debug)]
pub struct Archetype {
    pub archetype_specs: ArchetypeSpecs,
    pub entities: Vec<Entity>,
    pub components: Vec<Box<dyn ComponentColumn>>,
}

impl Archetype {
    pub fn new<T: Component>(archetype_specs: ArchetypeSpecs) -> Self {
        let mut archetype = Archetype {
            archetype_specs,
            entities: Vec::default(),
            components: Vec::with_capacity(1),
        };

        archetype
            .components
            .push(Box::new(RwLock::new(Vec::<T>::new())));

        archetype
    }

    pub fn new_from_archetype<T: Component>(
        archetype_specs: ArchetypeSpecs,
        from_archetype: &Archetype,
    ) -> Self {
        let mut archetype = Archetype {
            archetype_specs,
            entities: Vec::default(),
            components: Vec::with_capacity(1 + from_archetype.components.len()),
        };

        for c in from_archetype.components.iter() {
            archetype.components.push(c.new_same_type());
        }

        archetype
            .components
            .push(Box::new(RwLock::new(Vec::<T>::new())));

        archetype
    }

    pub fn new_from_component(
        archetype_specs: ArchetypeSpecs,
        from_components: &Vec<Box<dyn ComponentColumn>>,
    ) -> Self {
        let mut archetype = Archetype {
            archetype_specs,
            entities: Vec::default(),
            components: Vec::with_capacity(from_components.len()),
        };

        for c in from_components.iter() {
            archetype.components.push(c.new_same_type());
        }

        archetype
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn replace_component<T: Component>(
        &mut self,
        column_index: usize,
        row_index: usize,
        component: T,
    ) {
        let column = component_vec_to_mut(self.components.get_mut(column_index).unwrap().as_mut());
        column[row_index] = component;
    }

    pub fn push<T: Component>(&mut self, component_index: usize, t: T) {
        let component_column = component_vec_to_mut(&mut *self.components[component_index]);
        component_column.push(t)
    }

    /// Removes the component from an entity and pushes it to the other archetype
    /// The type does not need to be known to call this function.
    /// But the types of component_index and other_index need to match.
    pub fn migrate_component(
        &mut self,
        component_index: usize,
        entity_row: usize,
        other_archetype: &mut Archetype,
        other_index: usize,
    ) {
        self.components[component_index]
            .migrate(entity_row, &mut *other_archetype.components[other_index]);
    }
}

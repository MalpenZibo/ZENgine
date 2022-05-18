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

pub enum ComponentSearch {
    CurrentArchetype(Vec<(TypeId, usize, usize)>),
    NewArchetype(Vec<(TypeId, usize, usize)>),
}

#[derive(Debug)]
pub struct Archetype {
    pub archetype_specs: ArchetypeSpecs,
    pub entities: Vec<Entity>,
    pub components: Vec<Box<dyn ComponentColumn>>,
}

impl Archetype {
    pub fn root() -> Self {
        Archetype {
            archetype_specs: vec![],
            entities: Vec::default(),
            components: Vec::default(),
        }
    }

    pub fn new_from_component(
        archetype_specs: ArchetypeSpecs,
        from_components: Vec<Box<dyn ComponentColumn>>,
    ) -> Self {
        let mut archetype = Archetype {
            archetype_specs,
            entities: Vec::default(),
            components: Vec::with_capacity(from_components.len()),
        };

        for c in from_components.into_iter() {
            archetype.components.push(c);
        }

        archetype
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

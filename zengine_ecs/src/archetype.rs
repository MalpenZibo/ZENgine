use crate::{
    component::{component_vec_to_mut, Component, ComponentColumn},
    world::Entity,
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

#[derive(Default)]
pub struct Archetype {
    pub archetype_specs: ArchetypeSpecs,
    pub entities: Vec<Entity>,
    pub components: Vec<Box<dyn ComponentColumn>>,
}

impl Archetype {
    pub fn new<T: Component>(
        archetype_specs: ArchetypeSpecs,
        from_archetype: Option<&Archetype>,
    ) -> Self {
        let mut archetype = Archetype {
            archetype_specs,
            entities: Vec::default(),
            components: Vec::with_capacity(
                1 + from_archetype.map_or_else(|| 0, |a| a.components.len()),
            ),
        };

        if let Some(from_archetype) = from_archetype {
            for c in from_archetype.components.iter() {
                archetype.components.push(c.new_same_type());
            }
        }

        archetype
            .components
            .push(Box::new(RwLock::new(Vec::<T>::new())));

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

    pub fn extract_entity(
        &mut self,
        entity_row: usize,
    ) -> (Vec<Box<dyn Component>>, Option<Entity>) {
        self.entities.swap_remove(entity_row);

        (
            self.components
                .iter_mut()
                .map(|column| column.swap_remove(entity_row))
                .collect(),
            if self.entities.len() == 0 {
                None
            } else {
                Some(self.entities[entity_row])
            },
        )
    }

    pub fn push_components(&mut self, components: Vec<Box<dyn Component>>) {
        let mut index = 0;
        for component in components.into_iter() {
            self.components[index].push(component);
            index += 1;
        }
    }
}

use std::{
    any::TypeId,
    collections::HashMap,
    hash::{BuildHasherDefault, Hash, Hasher},
    ops::Deref,
};

use nohash_hasher::NoHashHasher;
use rustc_hash::FxHashMap;

use crate::{
    archetype::{calculate_archetype_id, Archetype, ArchetypeId},
    component::Component,
};

#[derive(Default)]
struct EntityGenerator {
    current: usize,
}
impl EntityGenerator {
    pub fn generate(&mut self) -> Entity {
        let entity = Entity(self.current);
        self.current += 1;

        entity
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Entity(usize);
impl Deref for Entity {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct Record {
    archetypeId: ArchetypeId,
    row: usize,
}

struct Edge {
    add: usize,
    remove: usize,
}

#[derive(Default)]
struct World {
    entity_generator: EntityGenerator,
    entities: FxHashMap<Entity, Record>,
    archetypes: HashMap<u64, Archetype, BuildHasherDefault<NoHashHasher<u64>>>,
}

impl World {
    pub fn spawn(&mut self) -> Entity {
        self.entity_generator.generate()
    }

    pub fn despawn(&mut self, entity: Entity) {}

    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        let component_id = TypeId::of::<T>();

        if let Some((new_archetype_id, new_row, replaced_entity, replaced_row)) =
            if let Some(record) = self.entities.get(&entity) {
                let mut archetype_specs = self
                    .archetypes
                    .get(&record.archetypeId)
                    .unwrap()
                    .archetype_specs
                    .clone();

                match archetype_specs.binary_search(&component_id) {
                    Ok(index) => {
                        // component already present in the entity archetype
                        // replace the old component with the new one
                        self.archetypes
                            .get_mut(&record.archetypeId)
                            .unwrap()
                            .replace_component(index, record.row, component);

                        None
                    }
                    Err(index) => {
                        // component not present in the entity archetype
                        // create the new archetype
                        archetype_specs.insert(index, component_id);
                        let new_archetype_id = calculate_archetype_id(&archetype_specs);

                        let (components, replaced_entity) = self
                            .archetypes
                            .get_mut(&record.archetypeId)
                            .unwrap()
                            .extract_entity(record.row);

                        let archetype_dest =
                            if let Some(archetype) = self.archetypes.get_mut(&new_archetype_id) {
                                archetype
                            } else {
                                self.archetypes.insert(
                                    new_archetype_id,
                                    Archetype::new::<T>(
                                        archetype_specs,
                                        Some(self.archetypes.get(&record.archetypeId).unwrap()),
                                    ),
                                );
                                self.archetypes.get_mut(&new_archetype_id).unwrap()
                            };

                        archetype_dest.push_components(components);

                        Some((
                            new_archetype_id,
                            archetype_dest.len() - 1,
                            replaced_entity,
                            record.row,
                        ))
                    }
                }
            } else {
                None
            }
        {
            self.entities.get_mut(&entity).map(|record| {
                record.archetypeId = new_archetype_id;
                record.row = new_row;
            });

            replaced_entity.map(|replaced_entity| {
                self.entities
                    .get_mut(&replaced_entity)
                    .map(|record| record.row = replaced_row)
            });
        }
    }

    pub fn remove_component<C: Component>(&mut self, entity: Entity) {
        let type_id = TypeId::of::<C>();
    }
}

/// A helper to get two mutable borrows from the same slice.
fn index_twice<T>(slice: &mut [T], first: usize, second: usize) -> (&mut T, &mut T) {
    if first < second {
        let (a, b) = slice.split_at_mut(second);
        (&mut a[first], &mut b[0])
    } else {
        let (a, b) = slice.split_at_mut(first);
        (&mut b[0], &mut a[second])
    }
}

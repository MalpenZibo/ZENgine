use std::{any::TypeId, collections::HashMap, hash::BuildHasherDefault};

use nohash_hasher::NoHashHasher;
use rustc_hash::FxHashMap;

use crate::{
    archetype::{calculate_archetype_id, Archetype},
    component::Component,
    entity::{Entity, EntityGenerator},
    ECSError,
};

#[derive(PartialEq, Debug)]
struct Record {
    archetype_index: usize,
    row: usize,
}

struct World {
    entity_generator: EntityGenerator,
    entity_record: FxHashMap<Entity, Record>,
    archetype_map: HashMap<u64, usize, BuildHasherDefault<NoHashHasher<u64>>>,
    archetypes: Vec<Archetype>,
}

impl Default for World {
    fn default() -> Self {
        let mut world = World {
            entity_generator: EntityGenerator::default(),
            entity_record: FxHashMap::default(),
            archetype_map: HashMap::default(),
            archetypes: Vec::default(),
        };

        let root_archetype = Archetype::root();
        let root_archetype_id = calculate_archetype_id(&root_archetype.archetype_specs);

        world.archetypes.push(root_archetype);
        world.archetype_map.insert(root_archetype_id, 0);

        world
    }
}

impl World {
    pub fn spawn(&mut self) -> Entity {
        let entity = self.entity_generator.generate();

        let root_archetype = self
            .archetypes
            .get_mut(0)
            .expect("root archetype should be present");
        root_archetype.entities.push(entity);
        self.entity_record.insert(
            entity,
            Record {
                archetype_index: 0,
                row: root_archetype.entities.len() - 1,
            },
        );

        entity
    }

    pub fn despawn(&mut self, entity: Entity) {}

    pub fn add_component<T: Component>(
        &mut self,
        entity: Entity,
        component: T,
    ) -> Result<(), ECSError> {
        let component_id = TypeId::of::<T>();

        enum EntitySearch {
            CurrentArchetype(usize, usize, usize),
            NewArchetype(usize, usize, usize),
        }

        match self.entity_record.get(&entity).map(|record| {
            self.archetypes
                .get(record.archetype_index)
                .expect("Wrong archetype index")
                .archetype_specs
                .binary_search(&component_id)
                .map(|index| {
                    EntitySearch::CurrentArchetype(record.archetype_index, index, record.row)
                })
                .unwrap_or_else(|err| {
                    EntitySearch::NewArchetype(record.archetype_index, err, record.row)
                })
        }) {
            Some(EntitySearch::CurrentArchetype(archetype_index, column_index, row_index)) => {
                // Entity present and component already in the current archetype
                // replace the old component with the new one
                self.archetypes
                    .get_mut(archetype_index)
                    .unwrap()
                    .replace_component(column_index, row_index, component);

                Ok(())
            }
            Some(EntitySearch::NewArchetype(
                source_archetype_index,
                new_column_insert_position,
                source_row_index,
            )) => {
                // calculate the new archetype specs and check if exist
                let mut destination_archetype_specs = self.archetypes[source_archetype_index]
                    .archetype_specs
                    .clone();
                destination_archetype_specs.insert(new_column_insert_position, component_id);
                let destination_archetype_id = calculate_archetype_id(&destination_archetype_specs);

                let destination_archetype_index = self
                    .archetype_map
                    .get(&destination_archetype_id)
                    .map(|index| *index)
                    .unwrap_or_else(|| {
                        let destination_archetype_index = self.archetypes.len();
                        self.archetypes.push(Archetype::new_from_archetype::<T>(
                            destination_archetype_specs,
                            self.archetypes.get(source_archetype_index).unwrap(),
                        ));
                        self.archetype_map
                            .insert(destination_archetype_id, destination_archetype_index);

                        destination_archetype_index
                    });

                // migrate component to the new archetype

                // index_twice lets us mutably borrow from the world twice.
                let (old_archetype, new_archetype) = index_twice(
                    &mut self.archetypes,
                    source_archetype_index,
                    destination_archetype_index,
                );

                old_archetype.entities.swap_remove(source_row_index);
                new_archetype.entities.push(entity);

                for i in 0..old_archetype.components.len() {
                    old_archetype.migrate_component(i, source_row_index, new_archetype, i);
                }

                new_archetype.push(new_column_insert_position, component);

                for i in new_column_insert_position..old_archetype.components.len() {
                    old_archetype.migrate_component(i, source_row_index, new_archetype, i);
                }

                // component migrated

                // update entity reference

                // get the entity tthat take the place of the old one
                old_archetype
                    .entities
                    .get(source_row_index)
                    .and_then(|entity| self.entity_record.get_mut(entity))
                    .map(|record| record.row = source_row_index);

                self.entity_record.get_mut(&entity).map(|record| {
                    record.archetype_index = destination_archetype_index;
                    record.row = new_archetype.entities.len() - 1;
                });

                Ok(())
            }
            None => Err(ECSError::EntityNotValid),
        }
    }

    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Result<(), ECSError> {
        let component_id = TypeId::of::<T>();

        enum EntitySearch {
            NewArchetype(usize, usize, usize),
            ComponentNotFound,
        };

        match self.entity_record.get(&entity).map(|record| {
            self.archetypes
                .get(record.archetype_index)
                .expect("Wrong archetype index")
                .archetype_specs
                .binary_search(&component_id)
                .map_or_else(
                    |_| EntitySearch::ComponentNotFound,
                    |column_index| {
                        EntitySearch::NewArchetype(record.archetype_index, column_index, record.row)
                    },
                )
        }) {
            Some(EntitySearch::NewArchetype(
                source_archetype_index,
                old_column_insert_position,
                source_row_index,
            )) => {
                // calculate the new archetype specs and check if exist
                let mut destination_archetype_specs = self.archetypes[source_archetype_index]
                    .archetype_specs
                    .clone();
                destination_archetype_specs.remove(old_column_insert_position);
                let destination_archetype_id = calculate_archetype_id(&destination_archetype_specs);

                let destination_archetype_index = self
                    .archetype_map
                    .get(&destination_archetype_id)
                    .map(|index| *index)
                    .unwrap_or_else(|| {
                        let destination_archetype_index = self.archetypes.len();
                        self.archetypes.push(Archetype::new_from_component(
                            destination_archetype_specs,
                            &self
                                .archetypes
                                .get(source_archetype_index)
                                .unwrap()
                                .components,
                        ));
                        self.archetype_map
                            .insert(destination_archetype_id, destination_archetype_index);

                        destination_archetype_index
                    });

                // migrate component to the new archetype

                // index_twice lets us mutably borrow from the world twice.
                let (old_archetype, new_archetype) = index_twice(
                    &mut self.archetypes,
                    source_archetype_index,
                    destination_archetype_index,
                );

                old_archetype.entities.swap_remove(source_row_index);
                new_archetype.entities.push(entity);

                for i in 0..old_column_insert_position {
                    old_archetype.migrate_component(i, source_row_index, new_archetype, i);
                }

                for i in old_column_insert_position + 1..old_archetype.components.len() {
                    old_archetype.migrate_component(i, source_row_index, new_archetype, i);
                }

                // component migrated

                // update entity reference

                // get the entity tthat take the place of the old one
                old_archetype
                    .entities
                    .get(source_row_index)
                    .and_then(|entity| self.entity_record.get_mut(entity))
                    .map(|record| record.row = source_row_index);

                self.entity_record.get_mut(&entity).map(|record| {
                    record.archetype_index = destination_archetype_index;
                    record.row = new_archetype.entities.len() - 1;
                });

                Ok(())
            }
            Some(EntitySearch::ComponentNotFound) => Err(ECSError::EntityDontHaveComponent),
            None => Err(ECSError::EntityNotValid),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Component1 {}
    impl Component for Component1 {}

    #[derive(Debug)]
    struct Component2 {}
    impl Component for Component2 {}

    #[derive(Debug)]
    struct Component3 {}
    impl Component for Component3 {}

    #[test]
    fn insert_component() {
        let mut world = World::default();

        let entity = world.spawn();

        world.add_component(entity, Component1 {}).unwrap();

        assert_eq!(
            world.entity_record.get(&entity),
            Some(&Record {
                archetype_index: 1,
                row: 0
            })
        );
        assert_eq!(world.archetypes.len(), 2);
    }

    #[test]
    fn insert_two_component_same_entity() {
        let mut world = World::default();

        let entity = world.spawn();

        world.add_component(entity, Component1 {}).unwrap();
        world.add_component(entity, Component2 {}).unwrap();

        assert_eq!(
            world.entity_record.get(&entity),
            Some(&Record {
                archetype_index: 2,
                row: 0
            })
        );
        assert_eq!(world.archetypes.len(), 3);
    }

    #[test]
    fn remove_component() {
        let mut world = World::default();

        let entity = world.spawn();

        world.add_component(entity, Component1 {}).unwrap();
        world.add_component(entity, Component2 {}).unwrap();
        world.remove_component::<Component2>(entity).unwrap();

        assert_eq!(
            world.entity_record.get(&entity),
            Some(&Record {
                archetype_index: 1,
                row: 0
            })
        );
        assert_eq!(world.archetypes.len(), 3);
    }
}

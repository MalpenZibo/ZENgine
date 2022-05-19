use std::{collections::HashMap, hash::BuildHasherDefault};

use nohash_hasher::NoHashHasher;
use rustc_hash::FxHashMap;

use crate::{
    archetype::{calculate_archetype_id, Archetype, ArchetypeSpecs},
    component::{ComponentBundle, ComponentColumn},
    entity::{Entity, EntityGenerator},
    ECSError,
};

#[derive(PartialEq, Debug)]
struct Record {
    archetype_index: usize,
    row: usize,
}

#[derive(Debug)]
pub struct World {
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
    pub fn spawn_without_component(&mut self) -> Entity {
        self.internal_spawn()
    }

    pub fn spawn<T: ComponentBundle>(&mut self, component_bundle: T) -> Entity {
        let entity = self.internal_spawn();

        self.add_component(entity, component_bundle)
            .expect("entity should be present");

        entity
    }

    fn internal_spawn(&mut self) -> Entity {
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

    pub fn despawn(&mut self, entity: Entity) -> Result<(), ECSError> {
        if let Some(record) = self.entity_record.get(&entity) {
            let archetype = self
                .archetypes
                .get_mut(record.archetype_index)
                .expect("archetype should be present");
            let row = record.row;

            archetype.entities.swap_remove(row);
            for c in archetype.components.iter_mut() {
                c.swap_remove(row);
            }

            // get the entity that take the place of the old one
            if let Some(record) = archetype
                .entities
                .get(record.row)
                .and_then(|entity| self.entity_record.get_mut(entity))
            {
                record.row = row
            };

            self.entity_record.remove(&entity);

            Ok(())
        } else {
            Err(ECSError::EntityNotValid)
        }
    }

    pub fn add_component<T: ComponentBundle>(
        &mut self,
        entity: Entity,
        component_bundle: T,
    ) -> Result<(), ECSError> {
        let component_ids = T::get_types();

        if let Some(record) = self.entity_record.get(&entity) {
            let archetype = self
                .archetypes
                .get(record.archetype_index)
                .expect("archetype should be present");

            let mut destination_archetype_specs = archetype.archetype_specs.clone();
            let mut new_archetype = false;

            for c_id in component_ids.iter() {
                if let Err(insert_index) = destination_archetype_specs.binary_search(c_id) {
                    destination_archetype_specs.insert(insert_index, *c_id);
                    new_archetype = true;
                }
            }
            let column_indexes: Vec<usize> = destination_archetype_specs
                .iter()
                .enumerate()
                .filter(|(_, a)| component_ids.iter().any(|b| **a == *b))
                .map(|(index, _)| index)
                .collect();

            if new_archetype {
                let source_archetype = record.archetype_index;
                let source_row = record.row;

                let destination_archetype_id = calculate_archetype_id(&destination_archetype_specs);

                let destination_archetype_index = self
                    .archetype_map
                    .get(&destination_archetype_id)
                    .copied()
                    .unwrap_or_else(|| {
                        let destination_archetype_index = self.archetypes.len();
                        let mut component_columns: Vec<Box<dyn ComponentColumn>> = self
                            .archetypes
                            .get(source_archetype)
                            .expect("source archetype should be present")
                            .components
                            .iter()
                            .map(|column| column.new_same_type())
                            .collect();

                        let mut components = T::get_component_columns();
                        for column_index in column_indexes.iter().rev() {
                            if *column_index > component_columns.len() {
                                component_columns.push(components.pop().unwrap());
                            } else {
                                component_columns.insert(*column_index, components.pop().unwrap());
                            }
                        }

                        self.archetypes.push(Archetype::new_from_component(
                            destination_archetype_specs,
                            component_columns,
                        ));
                        self.archetype_map
                            .insert(destination_archetype_id, destination_archetype_index);

                        destination_archetype_index
                    });

                // migrate component to the new archetype

                // index_twice lets us mutably borrow from the world twice.
                let (old_archetype, new_archetype) = index_twice(
                    &mut self.archetypes,
                    source_archetype,
                    destination_archetype_index,
                );

                old_archetype.entities.swap_remove(source_row);
                new_archetype.entities.push(entity);

                let mut last_index = 0;
                for column_index in column_indexes.iter() {
                    for i in last_index..*column_index {
                        old_archetype.migrate_component(i, source_row, new_archetype, i);
                    }

                    last_index = *column_index + 1;
                }

                for i in last_index..old_archetype.components.len() {
                    old_archetype.migrate_component(i, source_row, new_archetype, i);
                }

                component_bundle.inser_into(new_archetype, column_indexes);

                // component migrated

                // update entity reference

                // get the entity that take the place of the old one
                if let Some(record) = old_archetype
                    .entities
                    .get(record.row)
                    .and_then(|entity| self.entity_record.get_mut(entity))
                {
                    record.row = source_row
                }

                if let Some(record) = self.entity_record.get_mut(&entity) {
                    record.archetype_index = destination_archetype_index;
                    record.row = new_archetype.entities.len() - 1;
                }
            } else {
                component_bundle.replace_into(
                    self.archetypes
                        .get_mut(record.archetype_index)
                        .expect("target archetype should be present"),
                    record.row,
                    column_indexes,
                );
            }

            Ok(())
        } else {
            Err(ECSError::EntityNotValid)
        }
    }

    pub fn remove_component<T: ComponentBundle>(&mut self, entity: Entity) -> Result<(), ECSError> {
        let component_ids = T::get_types();

        if let Some(record) = self.entity_record.get(&entity) {
            let archetype = self
                .archetypes
                .get(record.archetype_index)
                .expect("archetype should be present");

            let mut column_indexes: Vec<usize> = Vec::default();

            for c_id in component_ids.iter() {
                if let Ok(index) = archetype.archetype_specs.binary_search(c_id) {
                    column_indexes.push(index)
                }
            }
            let (migrate_column_indexes, destination_archetype_specs): (
                Vec<usize>,
                ArchetypeSpecs,
            ) = archetype
                .archetype_specs
                .iter()
                .enumerate()
                .filter(|(_, c1)| !component_ids.iter().any(|c2| c2 == *c1))
                .map(|(index, c)| (index, *c))
                .unzip();

            let source_archetype = record.archetype_index;
            let source_row = record.row;

            let destination_archetype_id = calculate_archetype_id(&destination_archetype_specs);

            let destination_archetype_index = self
                .archetype_map
                .get(&destination_archetype_id)
                .copied()
                .unwrap_or_else(|| {
                    let destination_archetype_index = self.archetypes.len();
                    let component_columns: Vec<Box<dyn ComponentColumn>> = self
                        .archetypes
                        .get(source_archetype)
                        .expect("source archetype should be present")
                        .components
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| migrate_column_indexes.iter().any(|c| c == index))
                        .map(|(_, column)| column.new_same_type())
                        .collect();

                    self.archetypes.push(Archetype::new_from_component(
                        destination_archetype_specs,
                        component_columns,
                    ));
                    self.archetype_map
                        .insert(destination_archetype_id, destination_archetype_index);

                    destination_archetype_index
                });

            // migrate component to the new archetype

            // index_twice lets us mutably borrow from the world twice.
            let (old_archetype, new_archetype) = index_twice(
                &mut self.archetypes,
                source_archetype,
                destination_archetype_index,
            );

            old_archetype.entities.swap_remove(source_row);
            new_archetype.entities.push(entity);

            for (column_index, _) in migrate_column_indexes
                .iter()
                .enumerate()
                .take(new_archetype.components.len())
            {
                old_archetype.migrate_component(
                    migrate_column_indexes[column_index],
                    source_row,
                    new_archetype,
                    column_index,
                );
            }

            // component migrated

            // update entity reference

            // get the entity that take the place of the old one
            if let Some(record) = old_archetype
                .entities
                .get(record.row)
                .and_then(|entity| self.entity_record.get_mut(entity))
            {
                record.row = source_row;
            }

            if let Some(record) = self.entity_record.get_mut(&entity) {
                record.archetype_index = destination_archetype_index;
                record.row = new_archetype.entities.len() - 1;
            };

            Ok(())
        } else {
            Err(ECSError::EntityNotValid)
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
    use std::sync::RwLock;

    use crate::component::Component;

    use super::*;

    #[derive(Debug)]
    struct Component1 {}
    impl Component for Component1 {}

    #[derive(Debug)]
    struct Component2 {}
    impl Component for Component2 {}

    #[derive(Debug, PartialEq)]
    struct Component3 {
        data: u32,
    }
    impl Component for Component3 {}

    #[test]
    fn spawn_without_component() {
        let mut world = World::default();

        let entity = world.spawn_without_component();

        assert_eq!(
            world.entity_record.get(&entity),
            Some(&Record {
                archetype_index: 0,
                row: 0
            })
        );
        assert_eq!(world.archetypes.len(), 1);
    }

    #[test]
    fn spawn() {
        let mut world = World::default();

        let entity = world.spawn(Component1 {});

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
    fn despawn() {
        let mut world = World::default();

        let entity1 = world.spawn_without_component();
        world.add_component(entity1, Component1 {}).unwrap();

        let entity2 = world.spawn_without_component();
        world.add_component(entity2, Component1 {}).unwrap();

        let entity3 = world.spawn_without_component();
        world.add_component(entity3, Component1 {}).unwrap();

        world.despawn(entity2).unwrap();

        assert_eq!(
            world.entity_record.get(&entity1),
            Some(&Record {
                archetype_index: 1,
                row: 0
            })
        );
        assert_eq!(world.entity_record.get(&entity2), None);
        assert_eq!(
            world.entity_record.get(&entity3),
            Some(&Record {
                archetype_index: 1,
                row: 1
            })
        );
        assert_eq!(world.archetypes.len(), 2);
    }

    #[test]
    fn component_insert() {
        let mut world = World::default();

        let entity = world.spawn_without_component();

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
    fn multiple_component_insert() {
        let mut world = World::default();

        let entity = world.spawn_without_component();

        world
            .add_component(entity, (Component1 {}, Component3 { data: 3 }))
            .unwrap();

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
    fn component_replace() {
        let mut world = World::default();

        let entity = world.spawn_without_component();

        world
            .add_component(entity, Component3 { data: 32 })
            .unwrap();
        world
            .add_component(entity, Component3 { data: 42 })
            .unwrap();

        assert_eq!(
            world.entity_record.get(&entity),
            Some(&Record {
                archetype_index: 1,
                row: 0
            })
        );
        assert_eq!(
            world.archetypes[1].components[0]
                .to_any()
                .downcast_ref::<RwLock<Vec<Component3>>>()
                .unwrap()
                .read()
                .unwrap()[0],
            Component3 { data: 42 }
        );
        assert_eq!(world.archetypes.len(), 2);
    }

    #[test]
    fn insert_two_component_same_entity() {
        let mut world = World::default();

        let entity = world.spawn_without_component();

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

        let entity = world.spawn_without_component();

        world
            .add_component(entity, (Component1 {}, Component3 { data: 2 }))
            .unwrap();
        world.add_component(entity, Component2 {}).unwrap();
        world
            .remove_component::<(Component1, Component3)>(entity)
            .unwrap();

        assert_eq!(
            world.entity_record.get(&entity),
            Some(&Record {
                archetype_index: 3,
                row: 0
            })
        );
        assert_eq!(world.archetypes.len(), 4);
    }

    #[test]
    fn insert_multiple_entity_and_add_one_component() {
        let mut world = World::default();

        let entity1 = world.spawn_without_component();
        world.add_component(entity1, Component1 {}).unwrap();

        let entity2 = world.spawn_without_component();
        world.add_component(entity2, Component1 {}).unwrap();

        let entity3 = world.spawn_without_component();
        world.add_component(entity3, Component1 {}).unwrap();

        world.add_component(entity2, Component2 {}).unwrap();

        assert_eq!(
            world.entity_record.get(&entity1),
            Some(&Record {
                archetype_index: 1,
                row: 0
            })
        );
        assert_eq!(
            world.entity_record.get(&entity2),
            Some(&Record {
                archetype_index: 2,
                row: 0
            })
        );
        assert_eq!(
            world.entity_record.get(&entity3),
            Some(&Record {
                archetype_index: 1,
                row: 1
            })
        );
        assert_eq!(world.archetypes.len(), 3);
    }
}

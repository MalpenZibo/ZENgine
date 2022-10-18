use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    fmt::Debug,
    hash::BuildHasherDefault,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use nohash_hasher::NoHashHasher;
use rustc_hash::FxHashMap;

use crate::{
    archetype::{calculate_archetype_id, Archetype, ArchetypeSpecs},
    component::{ComponentBundle, ComponentColumn, InsertType},
    entity::{Entity, EntityGenerator},
    event::{EventCell, EventHandler},
    query::{QueryParameters, QueryRunner},
    resource::{Resource, ResourceCell, UnsendableResource, UnsendableResourceCell},
};

#[derive(PartialEq, Debug)]
struct Record {
    archetype_index: usize,
    row: usize,
}

/// Stores and exposes operations on entities, components, resources
///
/// # Entity and Components
/// Each [Entity] has a set of [Components](crate::Component) and each component can have
/// up to one instance of each component type.
/// Entity and components can be created, updated, removed and queried using a given World.
///
/// # Resources
/// A [Resource] is an unique instance of a given type that implement the [Resource] trait
/// and don't belong to a specific Entity.
/// A [UnsendableResource] are a particular resource that cannot be shared between thread and
/// can be accessed only by the main thread.
///
/// # Usage inside a system
/// To see how a system can interact with the data stored inside the World see the [system module](crate::system)
#[derive(Debug)]
pub struct World {
    pub(crate) entity_generator: EntityGenerator,
    entity_record: FxHashMap<Entity, Record>,
    archetype_map: HashMap<u64, usize, BuildHasherDefault<NoHashHasher<u64>>>,
    pub(crate) archetypes: Vec<Archetype>,
    resources: FxHashMap<TypeId, Box<dyn ResourceCell>>,
    unsendable_resources: FxHashMap<TypeId, Box<dyn UnsendableResourceCell>>,
    event_handlers: FxHashMap<TypeId, Box<dyn EventCell>>,
}

impl Default for World {
    fn default() -> Self {
        let mut world = World {
            entity_generator: EntityGenerator::default(),
            entity_record: FxHashMap::default(),
            archetype_map: HashMap::default(),
            archetypes: Vec::default(),
            resources: FxHashMap::default(),
            unsendable_resources: FxHashMap::default(),
            event_handlers: FxHashMap::default(),
        };

        let root_archetype = Archetype::root();
        let root_archetype_id = calculate_archetype_id(&root_archetype.archetype_specs);

        world.archetypes.push(root_archetype);
        world.archetype_map.insert(root_archetype_id, 0);

        world
    }
}

impl World {
    /// Creates an [Entity] without any components
    pub fn spawn_without_component(&mut self) -> Entity {
        self.internal_spawn()
    }

    /// Creates an [Entity] with a single component or with tuple of components
    ///
    /// # Example
    /// ```
    /// #[derive(Component, Debug)]
    /// struct ComponentA {
    ///     value: u32,
    /// }
    ///
    /// #[derive(Component, Debug)]
    /// struct ComponentB {
    ///     value: f32,
    /// }
    ///
    /// let mut world = World::default();
    /// world.spawn((ComponentA { value: 3 }, ComponentB { value: 1.0 }));
    /// ```
    pub fn spawn<T: ComponentBundle>(&mut self, component_bundle: T) -> Entity {
        let entity = self.internal_spawn();

        self.add_component(entity, component_bundle);

        entity
    }

    pub(crate) fn spawn_reserved<T: ComponentBundle>(
        &mut self,
        entity: Entity,
        component_bundle: T,
    ) {
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

        self.add_component(entity, component_bundle);
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

    /// Removes an Entity from the World
    pub fn despawn(&mut self, entity: Entity) {
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
        }
    }

    /// Adds a component or a tuple of components to an Entity
    pub fn add_component<T: ComponentBundle>(&mut self, entity: Entity, component_bundle: T) {
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

            enum ColumnType {
                Add(usize),
                Replace(usize, usize),
            }
            let mut columns: Vec<ColumnType> = Vec::default();
            for c_type in component_ids.iter() {
                match (
                    archetype.archetype_specs.iter().position(|c| c == c_type),
                    destination_archetype_specs.iter().position(|c| c == c_type),
                ) {
                    (Some(old_index), Some(new_index)) => {
                        // replace
                        columns.push(ColumnType::Replace(old_index, new_index))
                    }
                    (None, Some(new_index)) => {
                        // add
                        columns.push(ColumnType::Add(new_index));
                    }
                    _ => {}
                }
            }

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
                        let mut component_columns: Vec<(TypeId, Box<dyn ComponentColumn>)> = self
                            .archetypes
                            .get(source_archetype)
                            .expect("source archetype should be present")
                            .components
                            .iter()
                            .map(|column| column.new_same_type())
                            .collect();

                        component_columns.append(
                            &mut T::get_component_columns()
                                .into_iter()
                                .enumerate()
                                .filter_map(|(index, c)| {
                                    if let Some(ColumnType::Add(_)) = columns.get(index) {
                                        Some(c)
                                    } else {
                                        None
                                    }
                                })
                                .collect(),
                        );

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

                for (old_column_index, new_column_index) in new_archetype
                    .archetype_specs
                    .iter()
                    .enumerate()
                    .filter_map(|(index, c1)| {
                        if old_archetype.archetype_specs.iter().any(|c2| c1 == c2) {
                            Some(index)
                        } else {
                            None
                        }
                    })
                    .enumerate()
                    .collect::<Vec<(usize, usize)>>()
                {
                    old_archetype.migrate_component(
                        old_column_index,
                        source_row,
                        new_archetype,
                        new_column_index,
                    );
                }

                let new_row = new_archetype.entities.len() - 1;
                component_bundle.inser_into(
                    new_archetype,
                    columns
                        .into_iter()
                        .map(|column| match column {
                            ColumnType::Add(column_index) => (InsertType::Add, column_index),
                            ColumnType::Replace(_, new_index) => {
                                (InsertType::Replace(new_row), new_index)
                            }
                        })
                        .collect(),
                );

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
                    record.row = new_row;
                }
            } else {
                component_bundle.inser_into(
                    self.archetypes
                        .get_mut(record.archetype_index)
                        .expect("target archetype should be present"),
                    columns
                        .into_iter()
                        .filter_map(|column| match column {
                            ColumnType::Replace(_, new_index) => {
                                Some((InsertType::Replace(record.row), new_index))
                            }
                            _ => None,
                        })
                        .collect(),
                );
            }
        }
    }

    /// Removes a component or a tuple of components from an Entity
    pub fn remove_component<T: ComponentBundle>(&mut self, entity: Entity) {
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
                    let component_columns: Vec<(TypeId, Box<dyn ComponentColumn>)> = self
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
            }
        }
    }

    /// Queries entity and components from the World
    ///
    /// Returns a [QueryRunner] to fetch the data requested. A query runner has an internal
    /// cache to improve the performance of the query with subsequent calls
    ///
    /// # Example
    /// ## Query all entities that have at least the ComponentA
    /// ```
    /// #[derive(Component, Debug)]
    /// struct ComponentA {
    ///     value: u32,
    /// }
    ///
    /// #[derive(Component, Debug)]
    /// struct ComponentB {
    ///     value: f32,
    /// }
    ///
    /// let mut query = world.query::<(&ComponentA,)>();
    /// for c_a in query.run(&world).iter() {
    ///     println!("Component A: {:?}", c_a);
    /// }
    /// ```
    ///
    /// ## Query all entities that have at least the ComponentA and ComponentB
    /// with a mutable access to ComponentB adding the Entity Key in the result
    /// ```
    /// #[derive(Component, Debug)]
    /// struct ComponentA {
    ///     value: u32,
    /// }
    ///
    /// #[derive(Component, Debug)]
    /// struct ComponentB {
    ///     value: f32,
    /// }
    ///
    /// let mut query = world.query::<(Entity, &ComponentA, &mut ComponentB)>();
    /// for (e, c_a, c_b) in query.run(&world).iter_mut() {
    ///     c_b.value = 5.;
    ///     println!(
    ///         "Entity: {:?}, Component A: {:?}, Component B: {:?}",
    ///         e, c_a, c_b
    ///     );
    /// }
    /// ```
    pub fn query<T: QueryParameters>(&self) -> QueryRunner<T> {
        QueryRunner::default()
    }

    /// Gets a reference to the resource of the given type if it exists
    pub fn get_resource<T: Resource + 'static>(&self) -> Option<RwLockReadGuard<T>> {
        let type_id = TypeId::of::<T>();

        self.resources.get(&type_id).map(|r| {
            r.to_any()
                .downcast_ref::<RwLock<T>>()
                .expect("donwcasting error")
                .try_read()
                .expect("lock error")
        })
    }

    /// Gets a mutable reference to the resource of the given type if it exists
    pub fn get_mut_resource<T: Resource + 'static>(&self) -> Option<RwLockWriteGuard<T>> {
        let type_id = TypeId::of::<T>();

        self.resources.get(&type_id).map(|r| {
            r.to_any()
                .downcast_ref::<RwLock<T>>()
                .expect("donwcasting error")
                .try_write()
                .expect("lock error")
        })
    }

    /// Creates a new resource of the given value
    ///
    /// Resource are unique data of a given type so if you create a resource
    /// of a type that already exists you will overwrite any existing data
    pub fn create_resource<T: Resource + 'static>(&mut self, resource: T) {
        let type_id = TypeId::of::<T>();

        self.resources
            .insert(type_id, Box::new(RwLock::new(resource)));
    }

    /// Removes a resource of a given type and returns it if it exists
    pub fn remove_resource<T: Resource + 'static>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();

        let t = self.resources.remove(&type_id).unwrap();
        let t = Box::into_raw(t);
        let t = unsafe { Box::from_raw(t.cast::<RwLock<T>>()) };

        Some(t.into_inner().expect("lock error"))
    }

    /// Gets a reference to an unsendable resource of the given type if it exists
    pub fn get_unsendable_resource<T: UnsendableResource + 'static>(&self) -> Option<Ref<T>> {
        let type_id = TypeId::of::<T>();

        self.unsendable_resources.get(&type_id).map(|r| {
            r.to_any()
                .downcast_ref::<RefCell<T>>()
                .expect("donwcasting error")
                .try_borrow()
                .expect("lock error")
        })
    }

    /// Gets a mutable reference to an unsendable resource of the given type if it exists
    pub fn get_mut_unsendable_resource<T: UnsendableResource + 'static>(
        &self,
    ) -> Option<RefMut<T>> {
        let type_id = TypeId::of::<T>();

        self.unsendable_resources.get(&type_id).map(|r| {
            r.to_any()
                .downcast_ref::<RefCell<T>>()
                .expect("donwcasting error")
                .try_borrow_mut()
                .expect("lock error")
        })
    }

    /// Creates a new unsendable resource of the given value
    ///
    /// Unsendable resource are unique data of a given type so if you create an unsendable resource
    /// of a type that already exists you will overwrite any existing data
    pub fn create_unsendable_resource<T: UnsendableResource + 'static>(&mut self, resource: T) {
        let type_id = TypeId::of::<T>();

        self.unsendable_resources
            .insert(type_id, Box::new(RefCell::new(resource)));
    }

    /// Removes an unsendable resource of a given type and returns it if it exists
    pub fn remove_unsendable_resource<T: UnsendableResource + 'static>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();

        let t = self.unsendable_resources.remove(&type_id).unwrap();
        let t = Box::into_raw(t);
        let t = unsafe { Box::from_raw(t.cast::<RefCell<T>>()) };
        Some(t.into_inner())
    }

    /// Destroy a resource using its type id
    pub fn destroy_resource_with_type_id(&mut self, id: TypeId) {
        self.resources.remove(&id);
    }

    /// Destroy an unsendable resource using its type id
    pub fn destroy_unsendable_resource_with_type_id(&mut self, id: TypeId) {
        self.unsendable_resources.remove(&id);
    }

    /// Gets a reference to an EventHandler of a given type
    pub fn get_event_handler<T: Any + Debug>(&self) -> Option<RwLockReadGuard<EventHandler<T>>> {
        let type_id = TypeId::of::<EventHandler<T>>();

        self.event_handlers.get(&type_id).map(|e| {
            e.to_any()
                .downcast_ref::<RwLock<EventHandler<T>>>()
                .expect("donwcasting error")
                .try_read()
                .expect("lock error")
        })
    }

    /// Gets a mutable reference to an EventHandler of a given type
    pub fn get_mut_event_handler<T: Any + Debug>(
        &self,
    ) -> Option<RwLockWriteGuard<EventHandler<T>>> {
        let type_id = TypeId::of::<EventHandler<T>>();

        self.event_handlers.get(&type_id).map(|e| {
            e.to_any()
                .downcast_ref::<RwLock<EventHandler<T>>>()
                .expect("donwcasting error")
                .try_write()
                .expect("lock error")
        })
    }

    /// Create a new event handler of a given type
    ///
    /// EventHandlers are unique handler of a given type so if you create a EventHandler
    /// of a type that already exists you will overwrite any existing handler
    pub fn create_event_handler<T: Any + Debug>(&mut self) {
        let type_id = TypeId::of::<EventHandler<T>>();

        self.event_handlers
            .insert(type_id, Box::new(RwLock::new(EventHandler::<T>::default())));
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
    use std::{any::TypeId, sync::RwLock};

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

    #[derive(Debug)]
    struct Component4 {}
    impl Component for Component4 {}

    #[derive(Debug)]
    struct Component5 {}
    impl Component for Component5 {}

    #[derive(Debug)]
    struct Component6 {}
    impl Component for Component6 {}

    #[derive(Debug)]
    struct Component7 {}
    impl Component for Component7 {}

    #[derive(Debug)]
    struct Component8 {}
    impl Component for Component8 {}

    #[derive(Debug, PartialEq)]
    struct Resource1 {
        data: u32,
    }
    impl Resource for Resource1 {}

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
        world.add_component(entity1, Component1 {});

        let entity2 = world.spawn_without_component();
        world.add_component(entity2, Component1 {});

        let entity3 = world.spawn_without_component();
        world.add_component(entity3, Component1 {});

        world.despawn(entity2);

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

        world.add_component(entity, Component1 {});

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

        world.add_component(entity, (Component1 {}, Component3 { data: 3 }));

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

        world.add_component(entity, Component3 { data: 32 });
        world.add_component(entity, Component3 { data: 42 });

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

        world.add_component(entity, Component1 {});
        world.add_component(entity, Component2 {});

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

        world.add_component(
            entity,
            (
                Component1 {},
                Component3 { data: 2 },
                Component7 {},
                Component4 {},
                Component6 {},
                Component8 {},
            ),
        );
        world.remove_component::<(Component3, Component6, Component4)>(entity);

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
    fn insert_multiple_entity_and_add_one_component() {
        let mut world = World::default();

        let entity1 = world.spawn_without_component();
        world.add_component(entity1, Component1 {});

        let entity2 = world.spawn_without_component();
        world.add_component(entity2, Component1 {});

        let entity3 = world.spawn_without_component();
        world.add_component(entity3, Component1 {});

        world.add_component(entity2, Component2 {});

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

    #[test]
    fn insert_component_as_first() {
        let mut world = World::default();

        let entity = world.spawn_without_component();

        world.add_component(entity, Component1 {});
        world.add_component(entity, Component4 {});

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
    fn add_component_and_replace_old_one() {
        let mut world = World::default();

        let entity = world.spawn_without_component();

        world.add_component(
            entity,
            (Component1 {}, Component3 { data: 2 }, Component7 {}),
        );
        world.add_component(
            entity,
            (Component4 {}, Component3 { data: 42 }, Component8 {}),
        );

        assert_eq!(
            world.entity_record.get(&entity),
            Some(&Record {
                archetype_index: 2,
                row: 0
            })
        );
        let component3_index = world.archetypes[2]
            .archetype_specs
            .iter()
            .enumerate()
            .find_map(|(index, c_id)| {
                if *c_id == TypeId::of::<Component3>() {
                    Some(index)
                } else {
                    None
                }
            })
            .unwrap();

        assert_eq!(
            world.archetypes[2].components[component3_index]
                .to_any()
                .downcast_ref::<RwLock<Vec<Component3>>>()
                .unwrap()
                .read()
                .unwrap()[0],
            Component3 { data: 42 }
        );
        assert_eq!(world.archetypes.len(), 3);
    }

    #[test]
    fn resources() {
        let mut world = World::default();

        world.create_resource(Resource1 { data: 4 });

        {
            let res = world.get_resource::<Resource1>().unwrap();

            assert_eq!(res.data, 4);
        }

        {
            let mut res = world.get_mut_resource::<Resource1>().unwrap();
            res.data = 7;
        }

        {
            let res = world.get_resource::<Resource1>().unwrap();

            assert_eq!(res.data, 7);
        }
    }
}

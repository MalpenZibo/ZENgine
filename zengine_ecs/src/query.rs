use std::{
    any::TypeId,
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use crate::{archetype::Archetype, iterators::ChainedIterator, world::World};

pub struct QueryRunner<T> {
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T: QueryParameter<'a>> QueryRunner<T> {
    pub fn run(world: &'a World) -> Query<'a, T> {
        Query {
            data: T::fetch(world),
        }
    }
}

pub struct Query<'a, T: QueryParameter<'a>> {
    data: <T as QueryParameter<'a>>::FetchItem,
}

pub trait QueryParameter<'a> {
    type FetchItem;

    fn fetch(world: &'a World) -> Self::FetchItem;
}

impl<'a, T: 'static> QueryParameter<'a> for &T {
    type FetchItem = Vec<RwLockReadGuard<'a, Vec<T>>>;

    fn fetch(world: &'a World) -> Self::FetchItem {
        let mut result: Self::FetchItem = Vec::default();
        let type_id = TypeId::of::<T>();
        for a in world.archetypes.iter() {
            if Self::matches_archetype(a) {
                let index = a
                    .archetype_specs
                    .iter()
                    .position(|c| *c == type_id)
                    .unwrap();

                result.push(a.get(index).try_read().unwrap());
            }
        }

        result
    }
}

impl<'a, T: 'static> QueryParameter<'a> for &mut T {
    type FetchItem = Vec<RwLockWriteGuard<'a, Vec<T>>>;

    fn fetch(world: &'a World) -> Self::FetchItem {
        let mut result: Self::FetchItem = Vec::default();
        let type_id = TypeId::of::<T>();
        for a in world.archetypes.iter() {
            if Self::matches_archetype(a) {
                let index = a
                    .archetype_specs
                    .iter()
                    .position(|c| *c == type_id)
                    .unwrap();

                result.push(a.get(index).try_write().unwrap());
            }
        }

        result
    }
}

pub trait FetchFromArchetype<'a> {
    type FetchItem;

    fn matches_archetype(archetype: &Archetype) -> bool;

    fn fetch_from_archetype(archetype: &'a Archetype) -> Self::FetchItem;
}

impl<'a, T: 'static> FetchFromArchetype<'a> for &T {
    type FetchItem = RwLockReadGuard<'a, Vec<T>>;

    fn matches_archetype(archetype: &Archetype) -> bool {
        let type_id = TypeId::of::<T>();
        archetype.archetype_specs.iter().any(|c| *c == type_id)
    }

    fn fetch_from_archetype(archetype: &'a Archetype) -> Self::FetchItem {
        let type_id = TypeId::of::<T>();
        let index = archetype
            .archetype_specs
            .iter()
            .position(|c| *c == type_id)
            .unwrap();

        archetype.get(index).try_read().unwrap()
    }
}

impl<'a, T: 'static> FetchFromArchetype<'a> for &mut T {
    type FetchItem = RwLockWriteGuard<'a, Vec<T>>;

    fn matches_archetype(archetype: &Archetype) -> bool {
        let type_id = TypeId::of::<T>();
        archetype.archetype_specs.iter().any(|c| *c == type_id)
    }

    fn fetch_from_archetype(archetype: &'a Archetype) -> Self::FetchItem {
        let type_id = TypeId::of::<T>();
        let index = archetype
            .archetype_specs
            .iter()
            .position(|c| *c == type_id)
            .unwrap();

        archetype.get(index).try_write().unwrap()
    }
}

impl<'a, A: FetchFromArchetype<'a>, B: FetchFromArchetype<'a>> QueryParameter<'a> for (A, B) {
    type FetchItem = Vec<(A::FetchItem, B::FetchItem)>;

    fn fetch(world: &'a World) -> Self::FetchItem {
        let mut result: Self::FetchItem = Vec::default();
        for a in world.archetypes.iter() {
            if A::matches_archetype(&a) && B::matches_archetype(&a) {
                result.push((A::fetch_from_archetype(a), B::fetch_from_archetype(a)));
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {

    use crate::{component::Component, world::World};

    #[derive(Debug, PartialEq)]
    struct Test1 {}
    impl Component for Test1 {}

    #[derive(Debug)]
    struct Test2 {}
    impl Component for Test2 {}

    #[test]
    fn simple_query() {
        let mut world = World::default();

        world.spawn((Test1 {}, Test2 {}));
        world.spawn(Test1 {});

        let query = world.query::<&Test1>();

        println!("{:?}", query.data);

        assert_eq!(query.data.len(), 2);

        query.iter();

        for set in query.data.iter() {
            assert_eq!(set.len(), 1);
            for c in set.iter() {
                assert_eq!(*c, Test1 {})
            }
        }

        for e in query.data.iter() {
            for c in e.iter() {}
        }
    }

    #[test]
    fn tuple_query() {
        let mut world = World::default();

        world.spawn((Test1 {}, Test2 {}));
        world.spawn(Test1 {});

        let query = world.query::<(&mut Test1, &Test2)>();

        println!("{:?}", query.data);

        assert_eq!(query.data.len(), 1);

        // for set in query.data.iter() {
        //     assert_eq!(set.len(), 1);
        //     for c in set.iter() {
        //         assert_eq!(*c, Test1 {})
        //     }
        // }
    }
}

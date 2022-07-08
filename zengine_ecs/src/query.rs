use std::{
    any::TypeId,
    iter::{zip, Zip},
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use zengine_macro::{all_tuples, query_iter_for_tuple};

use crate::{archetype::Archetype, iterators::*, world::World};

pub trait FetchableQuery<T: QueryParameters> {
    fn fetch(world: &World) -> Self;
}

pub struct Query<'a, T: QueryParameters> {
    pub data: <T as QueryParameterFetch<'a>>::FetchItem,
}

pub struct QueryCache {
    last_archetypes_count: usize,
    matched_archetypes: Vec<(usize, Vec<usize>)>,
}

pub trait QueryParameters: for<'a> QueryParameterFetch<'a> {}

pub trait QueryParameter {
    type Item: for<'a> QueryParameterFetchFromArchetype<'a>;

    fn matches_archetype(archetype: &Archetype) -> bool;
}

pub trait QueryParameterFetch<'a> {
    type FetchItem;

    fn fetch(world: &'a World, cache: &mut Option<QueryCache>) -> Self::FetchItem;
}

pub trait QueryParameterFetchFromArchetype<'a> {
    type ArchetypeFetchItem;

    fn fetch_from_archetype(
        archetype: &'a Archetype,
        column_cache: Option<usize>,
    ) -> (Self::ArchetypeFetchItem, usize);
}

pub trait QueryIter<'a> {
    type Iter: Iterator;
    fn iter(&'a mut self) -> Self::Iter;
}

#[doc(hidden)]
pub struct ReadQueryParameterFetch<T> {
    phantom: std::marker::PhantomData<T>,
}

impl<T: 'static> QueryParameter for &T {
    type Item = ReadQueryParameterFetch<T>;

    fn matches_archetype(archetype: &Archetype) -> bool {
        let type_id = TypeId::of::<T>();
        archetype.archetype_specs.iter().any(|c| *c == type_id)
    }
}

impl<'a, T: 'static> QueryParameterFetch<'a> for ReadQueryParameterFetch<T> {
    type FetchItem = Vec<RwLockReadGuard<'a, Vec<T>>>;

    fn fetch(world: &'a World, cache: &mut Option<QueryCache>) -> Self::FetchItem {
        let mut result: Self::FetchItem = Vec::default();
        if let Some(cache) = cache {
            for (archetype, columns_vector) in cache
                .matched_archetypes
                .iter()
                .map(|(i, column_indexes)| (world.archetypes.get(*i).unwrap(), column_indexes))
            {
                result.push(archetype.get(columns_vector[0]).try_read().unwrap());
            }
        } else {
            let mut new_cache = QueryCache {
                last_archetypes_count: world.archetypes.len(),
                matched_archetypes: Vec::default(),
            };
            let type_id = TypeId::of::<T>();
            for (archetype_index, a) in world.archetypes.iter().enumerate() {
                if let Some(index) = a.archetype_specs.iter().position(|c| *c == type_id) {
                    new_cache
                        .matched_archetypes
                        .push((archetype_index, vec![index]));
                    result.push(a.get(index).try_read().unwrap());
                }
            }
            cache.replace(new_cache);
        }

        result
    }
}

impl<'a, T: 'static> QueryParameterFetchFromArchetype<'a> for ReadQueryParameterFetch<T> {
    type ArchetypeFetchItem = RwLockReadGuard<'a, Vec<T>>;

    fn fetch_from_archetype(
        archetype: &'a Archetype,
        column_cache: Option<usize>,
    ) -> (Self::ArchetypeFetchItem, usize) {
        if let Some(column) = column_cache {
            (archetype.get(column).try_read().unwrap(), column)
        } else {
            let type_id = TypeId::of::<T>();
            let index = archetype
                .archetype_specs
                .iter()
                .position(|c| *c == type_id)
                .unwrap();

            (archetype.get(index).try_read().unwrap(), index)
        }
    }
}

#[doc(hidden)]
pub struct WriteQueryParameterFetch<T> {
    phantom: std::marker::PhantomData<T>,
}

impl<T: 'static> QueryParameter for &mut T {
    type Item = WriteQueryParameterFetch<T>;

    fn matches_archetype(archetype: &Archetype) -> bool {
        let type_id = TypeId::of::<T>();
        archetype.archetype_specs.iter().any(|c| *c == type_id)
    }
}

impl<'a, T: 'static> QueryParameterFetch<'a> for WriteQueryParameterFetch<T> {
    type FetchItem = Vec<RwLockWriteGuard<'a, Vec<T>>>;

    fn fetch(world: &'a World, cache: &mut Option<QueryCache>) -> Self::FetchItem {
        let mut result: Self::FetchItem = Vec::default();
        if let Some(cache) = cache {
            for (archetype, columns_vector) in cache
                .matched_archetypes
                .iter()
                .map(|(i, column_indexes)| (world.archetypes.get(*i).unwrap(), column_indexes))
            {
                result.push(archetype.get(columns_vector[0]).try_write().unwrap());
            }
        } else {
            let mut new_cache = QueryCache {
                last_archetypes_count: world.archetypes.len(),
                matched_archetypes: Vec::default(),
            };
            let type_id = TypeId::of::<T>();
            for (archetype_index, a) in world.archetypes.iter().enumerate() {
                if let Some(index) = a.archetype_specs.iter().position(|c| *c == type_id) {
                    new_cache
                        .matched_archetypes
                        .push((archetype_index, vec![index]));
                    result.push(a.get(index).try_write().unwrap());
                }
            }
            cache.replace(new_cache);
        }

        result
    }
}

impl<'a, T: 'static> QueryParameterFetchFromArchetype<'a> for WriteQueryParameterFetch<T> {
    type ArchetypeFetchItem = RwLockWriteGuard<'a, Vec<T>>;

    fn fetch_from_archetype(
        archetype: &'a Archetype,
        column_cache: Option<usize>,
    ) -> (Self::ArchetypeFetchItem, usize) {
        if let Some(column) = column_cache {
            (archetype.get(column).try_write().unwrap(), column)
        } else {
            let type_id = TypeId::of::<T>();
            let index = archetype
                .archetype_specs
                .iter()
                .position(|c| *c == type_id)
                .unwrap();

            (archetype.get(index).try_write().unwrap(), index)
        }
    }
}

macro_rules! impl_query_parameters {
    () => {};
    ($ty: ident) => {
        impl<$ty: QueryParameter> QueryParameters for ($ty,) {}

        impl<'a, $ty: QueryParameter> QueryParameterFetch<'a> for ($ty,) {
            type FetchItem = Vec<<$ty::Item as QueryParameterFetchFromArchetype<'a>>::ArchetypeFetchItem>;

            fn fetch(world: &'a World, cache: &mut Option<QueryCache>) -> Self::FetchItem {
                let mut result: Self::FetchItem = Vec::default();
                if let Some(cache) = cache {
                    for (archetype, columns_vector) in cache
                        .matched_archetypes
                        .iter()
                        .map(|(i, column_indexes)| (world.archetypes.get(*i).unwrap(), column_indexes)) {
                        for c in columns_vector {
                            result.push( <$ty::Item as QueryParameterFetchFromArchetype<'a>>::fetch_from_archetype(archetype, Some(*c)).0);
                        }
                    }
                } else {
                    let mut new_cache = QueryCache {
                        last_archetypes_count: world.archetypes.len(),
                        matched_archetypes: Vec::default(),
                    };
                    for (archetype_index, a) in world.archetypes.iter().enumerate() {
                        if $ty::matches_archetype(&a) {
                            let (column, column_index) = <$ty::Item as QueryParameterFetchFromArchetype<'a>>::fetch_from_archetype(a, None);
                            new_cache.matched_archetypes.push((archetype_index, vec!(column_index)));
                            result.push(column);
                        }
                    }
                    cache.replace(new_cache);
                }

                result
            }
        }
    };
    ($($ty: ident),+) => {
        impl<$($ty: QueryParameter),*> QueryParameters for ($($ty,)*) {}

        impl<'a, $($ty: QueryParameter),*> QueryParameterFetch<'a> for ($($ty,)*) {
            type FetchItem = Vec<( $(<$ty::Item as QueryParameterFetchFromArchetype<'a>>::ArchetypeFetchItem),*)>;

            fn fetch(world: &'a World, cache: &mut Option<QueryCache>) -> Self::FetchItem {
                let mut result: Self::FetchItem = Vec::default();
                if let Some(cache) = cache {
                    for (archetype, columns_vector) in cache
                        .matched_archetypes
                        .iter()
                        .map(|(i, column_indexes)| (world.archetypes.get(*i).unwrap(), column_indexes)) {

                    let mut column_index_iter = columns_vector.iter();

                    let data = ($( {
                        let column_index = column_index_iter.next().unwrap();
                        <$ty::Item as QueryParameterFetchFromArchetype<'a>>::fetch_from_archetype(archetype, Some(*column_index)).0
                    }),*);

                    result.push(data);
                }
                } else {
                    let mut new_cache = QueryCache {
                        last_archetypes_count: world.archetypes.len(),
                        matched_archetypes: Vec::default(),
                    };
                    for (archetype_index, a) in world.archetypes.iter().enumerate() {
                        if $($ty::matches_archetype(&a))&&* {
                            let mut column_indexes = Vec::default();
                            let data = ($( {
                                let (column, column_index) = <$ty::Item as QueryParameterFetchFromArchetype<'a>>::fetch_from_archetype(a, None);
                                column_indexes.push(column_index);

                                column}
                            ),*);

                            new_cache.matched_archetypes.push((archetype_index, column_indexes));
                            result.push(data);
                        }
                    }
                }

                result
            }
        }
    };
}
all_tuples!(impl_query_parameters, 0, 14, P);

impl<'a, 'b, T: 'static> QueryIter<'b> for RwLockReadGuard<'a, Vec<T>> {
    type Iter = std::slice::Iter<'b, T>;
    fn iter(&'b mut self) -> Self::Iter {
        <[T]>::iter(self)
    }
}

impl<'a, 'b, T: 'static> QueryIter<'b> for RwLockWriteGuard<'a, Vec<T>> {
    type Iter = std::slice::IterMut<'b, T>;
    fn iter(&'b mut self) -> Self::Iter {
        <[T]>::iter_mut(self)
    }
}

query_iter_for_tuple!(14);

#[cfg(test)]
mod tests {

    use crate::{component::Component, query::Query, world::World};

    use super::QueryIter;

    #[derive(Debug, PartialEq)]
    struct Test1 {
        data: u32,
    }
    impl Component for Test1 {}

    #[derive(Debug)]
    struct Test2 {
        _data: u32,
    }
    impl Component for Test2 {}

    #[derive(Debug)]
    struct Test3 {
        data: u32,
    }
    impl Component for Test3 {}

    #[test]
    fn simple_query() {
        let mut world = World::default();

        world.spawn((Test1 { data: 3 }, Test2 { _data: 3 }));
        world.spawn(Test1 { data: 2 });

        let mut query: Query<(&Test1,)> = world.query::<(&Test1,)>(None);

        assert_eq!(query.iter().count(), 2);
    }

    #[test]
    fn tuple_query() {
        let mut world = World::default();

        world.spawn((Test1 { data: 3 }, Test2 { _data: 3 }));
        world.spawn(Test1 { data: 3 });

        let mut query = world.query::<(&Test1, &Test2)>(None);

        assert_eq!(query.iter().count(), 1);
    }

    #[test]
    fn tuple_with_mutable_query() {
        let mut world = World::default();

        world.spawn((Test1 { data: 3 }, Test2 { _data: 3 }));
        world.spawn((Test1 { data: 3 }, Test2 { _data: 2 }));
        world.spawn(Test1 { data: 3 });

        let mut query = world.query::<(&mut Test1, &Test2)>(None);

        assert_eq!(query.iter().count(), 2);

        for (a, _b) in query.iter() {
            a.data = 5;
        }

        for (a, _b) in query.iter() {
            assert_eq!(a.data, 5);
        }
    }

    #[test]
    fn tuple_with_2_mutable_query() {
        let mut world = World::default();

        world.spawn((Test1 { data: 3 }, Test2 { _data: 3 }, Test3 { data: 3 }));
        world.spawn(Test1 { data: 3 });
        world.spawn(Test3 { data: 3 });
        world.spawn((Test1 { data: 3 }, Test2 { _data: 3 }, Test3 { data: 3 }));

        let mut query = world.query::<(&mut Test1, &Test2, &mut Test3)>(None);
        assert_eq!(query.iter().count(), 2);

        for (a, _b, c) in query.iter() {
            a.data = 5;
            c.data = 7;
        }

        for (a, _b, c) in query.iter() {
            assert_eq!(a.data, 5);
            assert_eq!(c.data, 7);
        }
    }
}

use crate::{archetype::Archetype, component::Component, entity::Entity, world::World};
use std::{
    any::TypeId,
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use super::{query_iterators::*, QueryCache};
use zengine_macro::all_tuples;

#[doc(hidden)]
pub trait QueryParameters: for<'a> QueryParameterFetch<'a> {}

#[doc(hidden)]
pub trait QueryParameter {
    type Item: for<'a> QueryParameterFetchFromArchetype<'a>;

    fn matches_archetype(archetype: &Archetype) -> bool;
}

#[doc(hidden)]
pub trait QueryParameterFetch<'a> {
    type FetchItem;

    fn fetch(world: &'a World, cache: &mut Option<QueryCache>) -> Self::FetchItem;
}

#[doc(hidden)]
pub trait QueryParameterFetchFromArchetype<'a> {
    type ArchetypeFetchItem;

    fn fetch_from_archetype(
        archetype: &'a Archetype,
        column_cache: Option<Option<usize>>,
    ) -> (Self::ArchetypeFetchItem, Option<usize>);
}

#[doc(hidden)]
pub trait QueryIter<'a> {
    type Iter: Iterator;
    fn iter(&'a self) -> Self::Iter;
}

#[doc(hidden)]
pub trait QueryIterMut<'a> {
    type Iter: Iterator;
    fn iter_mut(&'a mut self) -> Self::Iter;
}

#[doc(hidden)]
pub struct ReadQueryParameterFetch<T> {
    phantom: std::marker::PhantomData<T>,
}

impl QueryParameter for Entity {
    type Item = ReadQueryParameterFetch<Entity>;

    fn matches_archetype(_archetype: &Archetype) -> bool {
        true
    }
}

impl<'a> QueryParameterFetch<'a> for ReadQueryParameterFetch<Entity> {
    type FetchItem = Vec<&'a Vec<Entity>>;

    fn fetch(world: &'a World, cache: &mut Option<QueryCache>) -> Self::FetchItem {
        if let Some(some_cache) = cache {
            if some_cache.last_archetypes_count != world.archetypes.len() {
                cache.take();
            }
        }

        let mut result: Self::FetchItem = Vec::default();
        if let Some(cache) = cache {
            for (archetype, _) in cache
                .matched_archetypes
                .iter()
                .map(|(i, column_indexes)| (world.archetypes.get(*i).unwrap(), column_indexes))
            {
                result.push(&archetype.entities);
            }
        } else {
            let mut new_cache = QueryCache {
                last_archetypes_count: world.archetypes.len(),
                matched_archetypes: Vec::default(),
            };
            for (archetype_index, a) in world.archetypes.iter().enumerate() {
                new_cache.matched_archetypes.push((archetype_index, vec![]));
                result.push(&a.entities);
            }
            cache.replace(new_cache);
        }

        result
    }
}

impl<'a> QueryParameterFetchFromArchetype<'a> for ReadQueryParameterFetch<Entity> {
    type ArchetypeFetchItem = &'a Vec<Entity>;

    fn fetch_from_archetype(
        archetype: &'a Archetype,
        _column_cache: Option<Option<usize>>,
    ) -> (Self::ArchetypeFetchItem, Option<usize>) {
        (&archetype.entities, Some(0))
    }
}

impl<T: Component + 'static> QueryParameter for &T {
    type Item = ReadQueryParameterFetch<T>;

    fn matches_archetype(archetype: &Archetype) -> bool {
        let type_id = TypeId::of::<T>();
        archetype.archetype_specs.iter().any(|c| *c == type_id)
    }
}

impl<'a, T: Component + 'static> QueryParameterFetch<'a> for ReadQueryParameterFetch<T> {
    type FetchItem = Vec<RwLockReadGuard<'a, Vec<T>>>;

    fn fetch(world: &'a World, cache: &mut Option<QueryCache>) -> Self::FetchItem {
        if let Some(some_cache) = cache {
            if some_cache.last_archetypes_count != world.archetypes.len() {
                cache.take();
            }
        }

        let mut result: Self::FetchItem = Vec::default();
        if let Some(cache) = cache {
            for (archetype, columns_vector) in cache
                .matched_archetypes
                .iter()
                .map(|(i, column_indexes)| (world.archetypes.get(*i).unwrap(), column_indexes))
            {
                result.push(
                    archetype
                        .get(
                            columns_vector[0].expect(
                                "Cache column for non Optional Parameter should not be None",
                            ),
                        )
                        .try_read()
                        .unwrap(),
                );
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
                        .push((archetype_index, vec![Some(index)]));
                    result.push(a.get(index).try_read().unwrap());
                }
            }
            cache.replace(new_cache);
        }

        result
    }
}

impl<'a, T: Component + 'static> QueryParameterFetchFromArchetype<'a>
    for ReadQueryParameterFetch<T>
{
    type ArchetypeFetchItem = RwLockReadGuard<'a, Vec<T>>;

    fn fetch_from_archetype(
        archetype: &'a Archetype,
        column_cache: Option<Option<usize>>,
    ) -> (Self::ArchetypeFetchItem, Option<usize>) {
        if let Some(column) = column_cache {
            let column =
                column.expect("Cache column for non Optional Parameter should not be None");
            (archetype.get(column).try_read().unwrap(), Some(column))
        } else {
            let type_id = TypeId::of::<T>();
            let index = archetype
                .archetype_specs
                .iter()
                .position(|c| *c == type_id)
                .unwrap();

            (archetype.get(index).try_read().unwrap(), Some(index))
        }
    }
}

#[doc(hidden)]
pub struct WriteQueryParameterFetch<T> {
    phantom: std::marker::PhantomData<T>,
}

impl<T: Component + 'static> QueryParameter for &mut T {
    type Item = WriteQueryParameterFetch<T>;

    fn matches_archetype(archetype: &Archetype) -> bool {
        let type_id = TypeId::of::<T>();
        archetype.archetype_specs.iter().any(|c| *c == type_id)
    }
}

impl<'a, T: Component + 'static> QueryParameterFetch<'a> for WriteQueryParameterFetch<T> {
    type FetchItem = Vec<RwLockWriteGuard<'a, Vec<T>>>;

    fn fetch(world: &'a World, cache: &mut Option<QueryCache>) -> Self::FetchItem {
        let mut result: Self::FetchItem = Vec::default();
        if let Some(cache) = cache {
            for (archetype, columns_vector) in cache
                .matched_archetypes
                .iter()
                .map(|(i, column_indexes)| (world.archetypes.get(*i).unwrap(), column_indexes))
            {
                result.push(
                    archetype
                        .get(
                            columns_vector[0].expect(
                                "Cache column for non Optional Parameter should not be None",
                            ),
                        )
                        .try_write()
                        .unwrap(),
                );
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
                        .push((archetype_index, vec![Some(index)]));
                    result.push(a.get(index).try_write().unwrap());
                }
            }
            cache.replace(new_cache);
        }

        result
    }
}

impl<'a, T: Component + 'static> QueryParameterFetchFromArchetype<'a>
    for WriteQueryParameterFetch<T>
{
    type ArchetypeFetchItem = RwLockWriteGuard<'a, Vec<T>>;

    fn fetch_from_archetype(
        archetype: &'a Archetype,
        column_cache: Option<Option<usize>>,
    ) -> (Self::ArchetypeFetchItem, Option<usize>) {
        if let Some(column) = column_cache {
            let column =
                column.expect("Cache column for non Optional Parameter should not be None");
            (archetype.get(column).try_write().unwrap(), Some(column))
        } else {
            let type_id = TypeId::of::<T>();
            let index = archetype
                .archetype_specs
                .iter()
                .position(|c| *c == type_id)
                .unwrap();

            (archetype.get(index).try_write().unwrap(), Some(index))
        }
    }
}

impl<T: Component + 'static> QueryParameter for Option<&T> {
    type Item = Option<ReadQueryParameterFetch<T>>;

    fn matches_archetype(_archetype: &Archetype) -> bool {
        true
    }
}

impl<'a, T: Component + 'static> QueryParameterFetch<'a> for Option<ReadQueryParameterFetch<T>> {
    type FetchItem = Vec<Option<RwLockReadGuard<'a, Vec<T>>>>;

    fn fetch(world: &'a World, cache: &mut Option<QueryCache>) -> Self::FetchItem {
        if let Some(some_cache) = cache {
            if some_cache.last_archetypes_count != world.archetypes.len() {
                cache.take();
            }
        }

        let mut result: Self::FetchItem = Vec::default();
        if let Some(cache) = cache {
            for (archetype, columns_vector) in cache
                .matched_archetypes
                .iter()
                .map(|(i, column_indexes)| (world.archetypes.get(*i).unwrap(), column_indexes))
            {
                match columns_vector[0] {
                    Some(column) => {
                        result.push(Some(archetype.get(column).try_read().unwrap()));
                    }
                    None => {
                        result.push(None);
                    }
                }
            }
        } else {
            let mut new_cache = QueryCache {
                last_archetypes_count: world.archetypes.len(),
                matched_archetypes: Vec::default(),
            };
            let type_id = TypeId::of::<T>();
            for (archetype_index, a) in world.archetypes.iter().enumerate() {
                match a.archetype_specs.iter().position(|c| *c == type_id) {
                    Some(column) => {
                        new_cache
                            .matched_archetypes
                            .push((archetype_index, vec![Some(column)]));
                        result.push(Some(a.get(column).try_read().unwrap()));
                    }
                    None => {
                        new_cache
                            .matched_archetypes
                            .push((archetype_index, vec![None]));
                        result.push(None);
                    }
                }
            }
            cache.replace(new_cache);
        }

        result
    }
}

impl<'a, T: Component + 'static> QueryParameterFetchFromArchetype<'a>
    for Option<ReadQueryParameterFetch<T>>
{
    type ArchetypeFetchItem = Option<RwLockReadGuard<'a, Vec<T>>>;

    fn fetch_from_archetype(
        archetype: &'a Archetype,
        column_cache: Option<Option<usize>>,
    ) -> (Self::ArchetypeFetchItem, Option<usize>) {
        if let Some(column) = column_cache {
            match column {
                Some(column) => (
                    Some(archetype.get(column).try_read().unwrap()),
                    Some(column),
                ),
                None => (None, None),
            }
        } else {
            let type_id = TypeId::of::<T>();
            match archetype.archetype_specs.iter().position(|c| *c == type_id) {
                Some(column) => (
                    Some(archetype.get(column).try_read().unwrap()),
                    Some(column),
                ),
                None => (None, None),
            }
        }
    }
}

impl<T: Component + 'static> QueryParameter for Option<&mut T> {
    type Item = Option<ReadQueryParameterFetch<T>>;

    fn matches_archetype(_archetype: &Archetype) -> bool {
        true
    }
}

impl<'a, T: Component + 'static> QueryParameterFetch<'a> for Option<WriteQueryParameterFetch<T>> {
    type FetchItem = Vec<Option<RwLockWriteGuard<'a, Vec<T>>>>;

    fn fetch(world: &'a World, cache: &mut Option<QueryCache>) -> Self::FetchItem {
        if let Some(some_cache) = cache {
            if some_cache.last_archetypes_count != world.archetypes.len() {
                cache.take();
            }
        }

        let mut result: Self::FetchItem = Vec::default();
        if let Some(cache) = cache {
            for (archetype, columns_vector) in cache
                .matched_archetypes
                .iter()
                .map(|(i, column_indexes)| (world.archetypes.get(*i).unwrap(), column_indexes))
            {
                match columns_vector[0] {
                    Some(column) => {
                        result.push(Some(archetype.get(column).try_write().unwrap()));
                    }
                    None => {
                        result.push(None);
                    }
                }
            }
        } else {
            let mut new_cache = QueryCache {
                last_archetypes_count: world.archetypes.len(),
                matched_archetypes: Vec::default(),
            };
            let type_id = TypeId::of::<T>();
            for (archetype_index, a) in world.archetypes.iter().enumerate() {
                match a.archetype_specs.iter().position(|c| *c == type_id) {
                    Some(column) => {
                        new_cache
                            .matched_archetypes
                            .push((archetype_index, vec![Some(column)]));
                        result.push(Some(a.get(column).try_write().unwrap()));
                    }
                    None => {
                        new_cache
                            .matched_archetypes
                            .push((archetype_index, vec![None]));
                        result.push(None);
                    }
                }
            }
            cache.replace(new_cache);
        }

        result
    }
}

impl<'a, T: Component + 'static> QueryParameterFetchFromArchetype<'a>
    for Option<WriteQueryParameterFetch<T>>
{
    type ArchetypeFetchItem = Option<RwLockWriteGuard<'a, Vec<T>>>;

    fn fetch_from_archetype(
        archetype: &'a Archetype,
        column_cache: Option<Option<usize>>,
    ) -> (Self::ArchetypeFetchItem, Option<usize>) {
        if let Some(column) = column_cache {
            match column {
                Some(column) => (
                    Some(archetype.get(column).try_write().unwrap()),
                    Some(column),
                ),
                None => (None, None),
            }
        } else {
            let type_id = TypeId::of::<T>();
            match archetype.archetype_specs.iter().position(|c| *c == type_id) {
                Some(column) => (
                    Some(archetype.get(column).try_write().unwrap()),
                    Some(column),
                ),
                None => (None, None),
            }
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

impl<'a, 'b> QueryIter<'b> for &'a Vec<Entity> {
    type Iter = std::slice::Iter<'b, Entity>;
    fn iter(&'b self) -> Self::Iter {
        <[Entity]>::iter(self)
    }
}

impl<'a, 'b, T: 'static> QueryIter<'b> for RwLockReadGuard<'a, Vec<T>> {
    type Iter = std::slice::Iter<'b, T>;
    fn iter(&'b self) -> Self::Iter {
        <[T]>::iter(self)
    }
}

impl<'a, 'b, T: 'static> QueryIter<'b> for RwLockWriteGuard<'a, Vec<T>> {
    type Iter = std::slice::Iter<'b, T>;
    fn iter(&'b self) -> Self::Iter {
        <[T]>::iter(self)
    }
}

impl<'a, 'b, T: 'static> QueryIter<'b> for Option<RwLockReadGuard<'a, Vec<T>>> {
    type Iter = OptionalIterator<std::slice::Iter<'b, T>>;
    fn iter(&'b self) -> Self::Iter {
        self.as_ref().map_or_else(
            || OptionalIterator::NoneIterator,
            |value| OptionalIterator::SomeIterator(<[T]>::iter(value)),
        )
    }
}

impl<'a, 'b, T: 'static> QueryIter<'b> for Option<RwLockWriteGuard<'a, Vec<T>>> {
    type Iter = OptionalIterator<std::slice::Iter<'b, T>>;
    fn iter(&'b self) -> Self::Iter {
        self.as_ref().map_or_else(
            || OptionalIterator::NoneIterator,
            |value| OptionalIterator::SomeIterator(<[T]>::iter(value)),
        )
    }
}

impl<'a, 'b> QueryIterMut<'b> for &'a Vec<Entity> {
    type Iter = std::slice::Iter<'b, Entity>;
    fn iter_mut(&'b mut self) -> Self::Iter {
        <[Entity]>::iter(self)
    }
}

impl<'a, 'b, T: 'static> QueryIterMut<'b> for RwLockReadGuard<'a, Vec<T>> {
    type Iter = std::slice::Iter<'b, T>;
    fn iter_mut(&'b mut self) -> Self::Iter {
        <[T]>::iter(self)
    }
}

impl<'a, 'b, T: 'static> QueryIterMut<'b> for RwLockWriteGuard<'a, Vec<T>> {
    type Iter = std::slice::IterMut<'b, T>;
    fn iter_mut(&'b mut self) -> Self::Iter {
        <[T]>::iter_mut(self)
    }
}

impl<'a, 'b, T: 'static> QueryIterMut<'b> for Option<RwLockReadGuard<'a, Vec<T>>> {
    type Iter = OptionalIterator<std::slice::Iter<'b, T>>;
    fn iter_mut(&'b mut self) -> Self::Iter {
        self.as_ref().map_or_else(
            || OptionalIterator::NoneIterator,
            |value| OptionalIterator::SomeIterator(<[T]>::iter(value)),
        )
    }
}

impl<'a, 'b, T: 'static> QueryIterMut<'b> for Option<RwLockWriteGuard<'a, Vec<T>>> {
    type Iter = OptionalIterator<std::slice::IterMut<'b, T>>;
    fn iter_mut(&'b mut self) -> Self::Iter {
        self.as_mut().map_or_else(
            || OptionalIterator::NoneIterator,
            |value| OptionalIterator::SomeIterator(<[T]>::iter_mut(value)),
        )
    }
}

use crate::{archetype::Archetype, component::Component, entity::Entity, world::World};
use std::{
    any::TypeId,
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use super::{query_iterators::*, QueryCache};
use zengine_macro::all_tuples;

#[doc(hidden)]
pub trait QueryParameters: for<'a> QueryParameterFetch<'a> + Send + Sync {}

pub trait ReadOnlyQueryParameters: for<'a> QueryParameterFetch<'a> + Send + Sync {}

#[doc(hidden)]
pub trait QueryParameter: Send + Sync {
    type Item: for<'a> QueryParameterFetchFromArchetype<'a> + Send + Sync;

    fn matches_archetype(archetype: &Archetype) -> bool;
}

pub trait ReadOnlyQueryParameter: QueryParameter {}

#[doc(hidden)]
pub trait QueryParameterFetch<'a>: Send + Sync {
    type FetchItem: Send + Sync;

    fn fetch(world: &'a World, cache: &mut Option<QueryCache>) -> Self::FetchItem;
}

#[doc(hidden)]
pub trait QueryParameterFetchFromArchetype<'a>: Send + Sync {
    type ArchetypeFetchItem: Send + Sync + std::fmt::Debug;

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

impl ReadOnlyQueryParameter for Entity {}

impl<'a> QueryParameterFetch<'a> for ReadQueryParameterFetch<Entity> {
    type FetchItem = Vec<&'a Vec<Entity>>;

    fn fetch(world: &'a World, cache: &mut Option<QueryCache>) -> Self::FetchItem {
        let mut result: Self::FetchItem = Vec::default();
        if let Some(cache) = cache {
            for (archetype, _) in cache
                .matched_archetypes
                .iter()
                .map(|(i, column_indexes)| (world.archetypes.get(*i).unwrap(), column_indexes))
            {
                if !archetype.entities.is_empty() {
                    result.push(&archetype.entities);
                }
            }
        } else {
            let mut new_cache = QueryCache {
                last_archetypes_count: world.archetypes.len(),
                matched_archetypes: Vec::default(),
            };
            for (archetype_index, a) in world.archetypes.iter().enumerate() {
                new_cache.matched_archetypes.push((archetype_index, vec![]));
                if !a.entities.is_empty() {
                    result.push(&a.entities);
                }
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

impl<T: Component + 'static> ReadOnlyQueryParameter for &T {}

pub struct QueryItem<'a, T>(Vec<RwLockReadGuard<'a, Vec<T>>>);
unsafe impl<'a, T> Send for QueryItem<'a, T> {}

impl<'a, T: Component + 'static> QueryParameterFetch<'a> for ReadQueryParameterFetch<T> {
    type FetchItem = QueryItem<'a, T>;

    fn fetch(world: &'a World, cache: &mut Option<QueryCache>) -> Self::FetchItem {
        let mut result: Self::FetchItem = QueryItem(Vec::default());
        if let Some(cache) = cache {
            for (archetype, columns_vector) in cache
                .matched_archetypes
                .iter()
                .map(|(i, column_indexes)| (world.archetypes.get(*i).unwrap(), column_indexes))
            {
                if !archetype.entities.is_empty() {
                    result.0.push(
                        archetype
                            .get(columns_vector[0].expect(
                                "Cache column for non Optional Parameter should not be None",
                            ))
                            .try_read()
                            .unwrap(),
                    );
                }
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
                    if !a.entities.is_empty() {
                        result.0.push(a.get(index).try_read().unwrap());
                    }
                }
            }
            cache.replace(new_cache);
        }

        result
    }
}

#[derive(Debug)]
pub struct ArchetypeItem<'a, T>(RwLockReadGuard<'a, Vec<T>>);
unsafe impl<'a, T> Send for ArchetypeItem<'a, T> {}

impl<'a, T: Component + 'static> QueryParameterFetchFromArchetype<'a>
    for ReadQueryParameterFetch<T>
{
    type ArchetypeFetchItem = ArchetypeItem<'a, T>;

    fn fetch_from_archetype(
        archetype: &'a Archetype,
        column_cache: Option<Option<usize>>,
    ) -> (Self::ArchetypeFetchItem, Option<usize>) {
        if let Some(column) = column_cache {
            let column =
                column.expect("Cache column for non Optional Parameter should not be None");
            (
                ArchetypeItem(archetype.get(column).try_read().unwrap()),
                Some(column),
            )
        } else {
            let type_id = TypeId::of::<T>();
            let index = archetype
                .archetype_specs
                .iter()
                .position(|c| *c == type_id)
                .unwrap();

            (
                ArchetypeItem(archetype.get(index).try_read().unwrap()),
                Some(index),
            )
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

pub struct QueryItemMut<'a, T>(Vec<RwLockWriteGuard<'a, Vec<T>>>);
unsafe impl<'a, T> Send for QueryItemMut<'a, T> {}

impl<'a, T: Component + 'static> QueryParameterFetch<'a> for WriteQueryParameterFetch<T> {
    type FetchItem = QueryItemMut<'a, T>;

    fn fetch(world: &'a World, cache: &mut Option<QueryCache>) -> Self::FetchItem {
        let mut result: Self::FetchItem = QueryItemMut(Vec::default());
        if let Some(cache) = cache {
            for (archetype, columns_vector) in cache
                .matched_archetypes
                .iter()
                .map(|(i, column_indexes)| (world.archetypes.get(*i).unwrap(), column_indexes))
            {
                if !archetype.entities.is_empty() {
                    result.0.push(
                        archetype
                            .get(columns_vector[0].expect(
                                "Cache column for non Optional Parameter should not be None",
                            ))
                            .try_write()
                            .unwrap(),
                    );
                }
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
                    if !a.entities.is_empty() {
                        result.0.push(a.get(index).try_write().unwrap());
                    }
                }
            }
            cache.replace(new_cache);
        }

        result
    }
}

#[derive(Debug)]
pub struct ArchetypeItemMut<'a, T>(RwLockWriteGuard<'a, Vec<T>>);
unsafe impl<'a, T> Send for ArchetypeItemMut<'a, T> {}

impl<'a, T: Component + 'static> QueryParameterFetchFromArchetype<'a>
    for WriteQueryParameterFetch<T>
{
    type ArchetypeFetchItem = ArchetypeItemMut<'a, T>;

    fn fetch_from_archetype(
        archetype: &'a Archetype,
        column_cache: Option<Option<usize>>,
    ) -> (Self::ArchetypeFetchItem, Option<usize>) {
        if let Some(column) = column_cache {
            let column =
                column.expect("Cache column for non Optional Parameter should not be None");
            (
                ArchetypeItemMut(archetype.get(column).try_write().unwrap()),
                Some(column),
            )
        } else {
            let type_id = TypeId::of::<T>();
            let index = archetype
                .archetype_specs
                .iter()
                .position(|c| *c == type_id)
                .unwrap();

            (
                ArchetypeItemMut(archetype.get(index).try_write().unwrap()),
                Some(index),
            )
        }
    }
}

impl<T: Component + 'static> QueryParameter for Option<&T> {
    type Item = Option<ReadQueryParameterFetch<T>>;

    fn matches_archetype(_archetype: &Archetype) -> bool {
        true
    }
}

impl<T: Component + 'static> ReadOnlyQueryParameter for Option<&T> {}

pub struct QueryOptionalItem<'a, T>(Vec<Option<RwLockReadGuard<'a, Vec<T>>>>);
unsafe impl<'a, T> Send for QueryOptionalItem<'a, T> {}

impl<'a, T: Component + 'static> QueryParameterFetch<'a> for Option<ReadQueryParameterFetch<T>> {
    type FetchItem = QueryOptionalItem<'a, T>;

    fn fetch(world: &'a World, cache: &mut Option<QueryCache>) -> Self::FetchItem {
        let mut result: Self::FetchItem = QueryOptionalItem(Vec::default());
        if let Some(cache) = cache {
            for (archetype, columns_vector) in cache
                .matched_archetypes
                .iter()
                .map(|(i, column_indexes)| (world.archetypes.get(*i).unwrap(), column_indexes))
            {
                if !archetype.entities.is_empty() {
                    match columns_vector[0] {
                        Some(column) => {
                            result
                                .0
                                .push(Some(archetype.get(column).try_read().unwrap()));
                        }
                        None => {
                            result.0.push(None);
                        }
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
                        if !a.entities.is_empty() {
                            result.0.push(Some(a.get(column).try_read().unwrap()));
                        }
                    }
                    None => {
                        new_cache
                            .matched_archetypes
                            .push((archetype_index, vec![None]));
                        if !a.entities.is_empty() {
                            result.0.push(None);
                        }
                    }
                }
            }
            cache.replace(new_cache);
        }

        result
    }
}

#[derive(Debug)]
pub struct ArchetypeOptionalItem<'a, T>(Option<RwLockReadGuard<'a, Vec<T>>>);
unsafe impl<'a, T> Send for ArchetypeOptionalItem<'a, T> {}

impl<'a, T: Component + 'static> QueryParameterFetchFromArchetype<'a>
    for Option<ReadQueryParameterFetch<T>>
{
    type ArchetypeFetchItem = ArchetypeOptionalItem<'a, T>;

    fn fetch_from_archetype(
        archetype: &'a Archetype,
        column_cache: Option<Option<usize>>,
    ) -> (Self::ArchetypeFetchItem, Option<usize>) {
        if let Some(column) = column_cache {
            match column {
                Some(column) => (
                    ArchetypeOptionalItem(Some(archetype.get(column).try_read().unwrap())),
                    Some(column),
                ),
                None => (ArchetypeOptionalItem(None), None),
            }
        } else {
            let type_id = TypeId::of::<T>();
            match archetype.archetype_specs.iter().position(|c| *c == type_id) {
                Some(column) => (
                    ArchetypeOptionalItem(Some(archetype.get(column).try_read().unwrap())),
                    Some(column),
                ),
                None => (ArchetypeOptionalItem(None), None),
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

pub struct QueryOptionalItemMut<'a, T>(Vec<Option<RwLockWriteGuard<'a, Vec<T>>>>);
unsafe impl<'a, T> Send for QueryOptionalItemMut<'a, T> {}

impl<'a, T: Component + 'static> QueryParameterFetch<'a> for Option<WriteQueryParameterFetch<T>> {
    type FetchItem = QueryOptionalItemMut<'a, T>;

    fn fetch(world: &'a World, cache: &mut Option<QueryCache>) -> Self::FetchItem {
        let mut result: Self::FetchItem = QueryOptionalItemMut(Vec::default());
        if let Some(cache) = cache {
            for (archetype, columns_vector) in cache
                .matched_archetypes
                .iter()
                .map(|(i, column_indexes)| (world.archetypes.get(*i).unwrap(), column_indexes))
            {
                if !archetype.entities.is_empty() {
                    match columns_vector[0] {
                        Some(column) => {
                            result
                                .0
                                .push(Some(archetype.get(column).try_write().unwrap()));
                        }
                        None => {
                            result.0.push(None);
                        }
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
                        if !a.entities.is_empty() {
                            result.0.push(Some(a.get(column).try_write().unwrap()));
                        }
                    }
                    None => {
                        new_cache
                            .matched_archetypes
                            .push((archetype_index, vec![None]));
                        if !a.entities.is_empty() {
                            result.0.push(None);
                        }
                    }
                }
            }
            cache.replace(new_cache);
        }

        result
    }
}

#[derive(Debug)]
pub struct ArchetypeOptionalItemMut<'a, T>(Option<RwLockWriteGuard<'a, Vec<T>>>);
unsafe impl<'a, T> Send for ArchetypeOptionalItemMut<'a, T> {}

impl<'a, T: Component + 'static> QueryParameterFetchFromArchetype<'a>
    for Option<WriteQueryParameterFetch<T>>
{
    type ArchetypeFetchItem = ArchetypeOptionalItemMut<'a, T>;

    fn fetch_from_archetype(
        archetype: &'a Archetype,
        column_cache: Option<Option<usize>>,
    ) -> (Self::ArchetypeFetchItem, Option<usize>) {
        if let Some(column) = column_cache {
            match column {
                Some(column) => (
                    ArchetypeOptionalItemMut(Some(archetype.get(column).try_write().unwrap())),
                    Some(column),
                ),
                None => (ArchetypeOptionalItemMut(None), None),
            }
        } else {
            let type_id = TypeId::of::<T>();
            match archetype.archetype_specs.iter().position(|c| *c == type_id) {
                Some(column) => (
                    ArchetypeOptionalItemMut(Some(archetype.get(column).try_write().unwrap())),
                    Some(column),
                ),
                None => (ArchetypeOptionalItemMut(None), None),
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
                            if !archetype.entities.is_empty() {
                                result.push(<$ty::Item as QueryParameterFetchFromArchetype<'a>>::fetch_from_archetype(archetype, Some(*c)).0);
                            }
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
                            if !a.entities.is_empty() {
                                result.push(column);
                            }
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

                    if !archetype.entities.is_empty() {
                        let mut column_index_iter = columns_vector.iter();

                        let data = ($( {
                            let column_index = column_index_iter.next().unwrap();
                            <$ty::Item as QueryParameterFetchFromArchetype<'a>>::fetch_from_archetype(archetype, Some(*column_index)).0
                        }),*);

                        result.push(data);
                    }
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

                                    column
                                }
                            ),*);

                            new_cache.matched_archetypes.push((archetype_index, column_indexes));
                            if !a.entities.is_empty() {
                                result.push(data);
                            }
                        }
                    }
                }

                result
            }
        }
    };
}
all_tuples!(impl_query_parameters, 0, 14, P);

macro_rules! impl_readonly_query_parameters {
    () => {};
    ($ty: ident) => {
        impl<$ty: ReadOnlyQueryParameter> ReadOnlyQueryParameters for ($ty,) {}
    };
    ($($ty: ident),+) => {
        impl<$($ty: ReadOnlyQueryParameter),*> ReadOnlyQueryParameters for ($($ty,)*) {}
    };
}
all_tuples!(impl_readonly_query_parameters, 0, 14, P);

impl<'a, 'b> QueryIter<'b> for &'a Vec<Entity> {
    type Iter = std::slice::Iter<'b, Entity>;
    fn iter(&'b self) -> Self::Iter {
        <[Entity]>::iter(self)
    }
}

impl<'a, 'b, T: 'static> QueryIter<'b> for ArchetypeItem<'a, T> {
    type Iter = std::slice::Iter<'b, T>;
    fn iter(&'b self) -> Self::Iter {
        <[T]>::iter(&self.0)
    }
}

impl<'a, 'b, T: 'static> QueryIter<'b> for ArchetypeItemMut<'a, T> {
    type Iter = std::slice::Iter<'b, T>;
    fn iter(&'b self) -> Self::Iter {
        <[T]>::iter(&self.0)
    }
}

impl<'a, 'b, T: 'static> QueryIter<'b> for ArchetypeOptionalItem<'a, T> {
    type Iter = OptionalIterator<std::slice::Iter<'b, T>>;
    fn iter(&'b self) -> Self::Iter {
        self.0.as_ref().map_or_else(
            || OptionalIterator::NoneIterator,
            |value| OptionalIterator::SomeIterator(<[T]>::iter(value)),
        )
    }
}

impl<'a, 'b, T: 'static> QueryIter<'b> for ArchetypeOptionalItemMut<'a, T> {
    type Iter = OptionalIterator<std::slice::Iter<'b, T>>;
    fn iter(&'b self) -> Self::Iter {
        self.0.as_ref().map_or_else(
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

impl<'a, 'b, T: 'static> QueryIterMut<'b> for ArchetypeItem<'a, T> {
    type Iter = std::slice::Iter<'b, T>;
    fn iter_mut(&'b mut self) -> Self::Iter {
        <[T]>::iter(&self.0)
    }
}

impl<'a, 'b, T: 'static> QueryIterMut<'b> for ArchetypeItemMut<'a, T> {
    type Iter = std::slice::IterMut<'b, T>;
    fn iter_mut(&'b mut self) -> Self::Iter {
        <[T]>::iter_mut(&mut self.0)
    }
}

impl<'a, 'b, T: 'static> QueryIterMut<'b> for ArchetypeOptionalItem<'a, T> {
    type Iter = OptionalIterator<std::slice::Iter<'b, T>>;
    fn iter_mut(&'b mut self) -> Self::Iter {
        self.0.as_ref().map_or_else(
            || OptionalIterator::NoneIterator,
            |value| OptionalIterator::SomeIterator(<[T]>::iter(value)),
        )
    }
}

impl<'a, 'b, T: 'static> QueryIterMut<'b> for ArchetypeOptionalItemMut<'a, T> {
    type Iter = OptionalIterator<std::slice::IterMut<'b, T>>;
    fn iter_mut(&'b mut self) -> Self::Iter {
        self.0.as_mut().map_or_else(
            || OptionalIterator::NoneIterator,
            |value| OptionalIterator::SomeIterator(<[T]>::iter_mut(value)),
        )
    }
}

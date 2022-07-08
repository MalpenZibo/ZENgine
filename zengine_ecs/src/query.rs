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

pub trait QueryParameters: for<'a> QueryParameterFetch<'a> {}

pub trait QueryParameter {
    type Item: for<'a> QueryParameterFetch<'a>;

    fn matches_archetype(archetype: &Archetype) -> bool;
}

pub trait QueryParameterFetch<'a> {
    type FetchItem;
    type ArchetypeFetchItem;

    fn fetch(world: &'a World) -> Self::FetchItem;

    fn fetch_from_archetype(archetype: &'a Archetype) -> Self::ArchetypeFetchItem;
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
    type FetchItem = Vec<Self::ArchetypeFetchItem>;
    type ArchetypeFetchItem = RwLockReadGuard<'a, Vec<T>>;

    fn fetch_from_archetype(archetype: &'a Archetype) -> Self::ArchetypeFetchItem {
        let type_id = TypeId::of::<T>();
        let index = archetype
            .archetype_specs
            .iter()
            .position(|c| *c == type_id)
            .unwrap();

        archetype.get(index).try_read().unwrap()
    }

    fn fetch(world: &'a World) -> Self::FetchItem {
        let mut result: Self::FetchItem = Vec::default();
        let type_id = TypeId::of::<T>();
        for a in world.archetypes.iter() {
            if let Some(index) = a.archetype_specs.iter().position(|c| *c == type_id) {
                result.push(a.get(index).try_read().unwrap());
            }
        }

        result
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
    type FetchItem = Vec<Self::ArchetypeFetchItem>;
    type ArchetypeFetchItem = RwLockWriteGuard<'a, Vec<T>>;

    fn fetch_from_archetype(archetype: &'a Archetype) -> Self::ArchetypeFetchItem {
        let type_id = TypeId::of::<T>();
        let index = archetype
            .archetype_specs
            .iter()
            .position(|c| *c == type_id)
            .unwrap();

        archetype.get(index).try_write().unwrap()
    }

    fn fetch(world: &'a World) -> Self::FetchItem {
        let mut result: Self::FetchItem = Vec::default();
        let type_id = TypeId::of::<T>();
        for a in world.archetypes.iter() {
            if let Some(index) = a.archetype_specs.iter().position(|c| *c == type_id) {
                result.push(a.get(index).try_write().unwrap());
            }
        }

        result
    }
}

macro_rules! impl_query_parameters {
    () => {};
    ($ty: ident) => {
        impl<$ty: QueryParameter> QueryParameters for ($ty,) {}

        impl<'a, $ty: QueryParameter> QueryParameterFetch<'a> for ($ty,) {
            type FetchItem = Vec<Self::ArchetypeFetchItem>;
            type ArchetypeFetchItem = <$ty::Item as QueryParameterFetch<'a>>::ArchetypeFetchItem;

            fn fetch(world: &'a World) -> Self::FetchItem {
                let mut result: Self::FetchItem = Vec::default();
                for a in world.archetypes.iter() {
                    if $ty::matches_archetype(&a) {
                        result.push(Self::fetch_from_archetype(a));
                    }
                }

                result
            }

            fn fetch_from_archetype(archetype: &'a Archetype) -> Self::ArchetypeFetchItem {
                <$ty::Item as QueryParameterFetch<'a>>::fetch_from_archetype(archetype)
            }
        }
    };
    ($($ty: ident),+) => {
        impl<$($ty: QueryParameter),*> QueryParameters for ($($ty,)*) {}

        impl<'a, $($ty: QueryParameter),*> QueryParameterFetch<'a> for ($($ty,)*) {
            type FetchItem = Vec<Self::ArchetypeFetchItem>;
            type ArchetypeFetchItem = ( $(<$ty::Item as QueryParameterFetch<'a>>::ArchetypeFetchItem),*);

            fn fetch(world: &'a World) -> Self::FetchItem {
                let mut result: Self::FetchItem = Vec::default();
                for a in world.archetypes.iter() {
                    if $($ty::matches_archetype(&a))&&* {
                        result.push(Self::fetch_from_archetype(a));
                    }
                }

                result
            }

            fn fetch_from_archetype(archetype: &'a Archetype) -> Self::ArchetypeFetchItem {
                (
                    $(<$ty::Item as QueryParameterFetch<'a>>::fetch_from_archetype(archetype)),*
                )
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

        let mut query: Query<(&Test1,)> = world.query::<(&Test1,)>();

        assert_eq!(query.iter().count(), 2);
    }

    #[test]
    fn tuple_query() {
        let mut world = World::default();

        world.spawn((Test1 { data: 3 }, Test2 { _data: 3 }));
        world.spawn(Test1 { data: 3 });

        let mut query = world.query::<(&Test1, &Test2)>();

        assert_eq!(query.iter().count(), 1);
    }

    #[test]
    fn tuple_with_mutable_query() {
        let mut world = World::default();

        world.spawn((Test1 { data: 3 }, Test2 { _data: 3 }));
        world.spawn((Test1 { data: 3 }, Test2 { _data: 2 }));
        world.spawn(Test1 { data: 3 });

        let mut query = world.query::<(&mut Test1, &Test2)>();

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

        let mut query = world.query::<(&mut Test1, &Test2, &mut Test3)>();
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

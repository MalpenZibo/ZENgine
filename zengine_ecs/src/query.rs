use std::{
    any::TypeId,
    iter::{zip, Zip},
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use crate::{archetype::Archetype, iterators::QueryIterator, iterators::Zip3, world::World};

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

pub trait GetItem<'a> {
    type Item;

    fn get_item(&'a mut self, row: usize) -> Option<Self::Item>;
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
impl_query_parameters!(A);
impl_query_parameters!(A, B);
impl_query_parameters!(A, B, C);
impl_query_parameters!(A, B, C, D);
impl_query_parameters!(A, B, C, D, E);
impl_query_parameters!(A, B, C, D, E, F);
impl_query_parameters!(A, B, C, D, E, F, G);
impl_query_parameters!(A, B, C, D, E, F, G, H);
impl_query_parameters!(A, B, C, D, E, F, G, H, I);
impl_query_parameters!(A, B, C, D, E, F, G, H, I, J);
impl_query_parameters!(A, B, C, D, E, F, G, H, I, J, K);
impl_query_parameters!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_query_parameters!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_query_parameters!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_query_parameters!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_query_parameters!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_query_parameters!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_query_parameters!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_query_parameters!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_query_parameters!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_query_parameters!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
impl_query_parameters!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
impl_query_parameters!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
impl_query_parameters!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
impl_query_parameters!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
impl_query_parameters!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
);

impl<'a, 'b, T: 'static> QueryIter<'b> for RwLockReadGuard<'a, Vec<T>> {
    type Iter = std::slice::Iter<'b, T>;
    fn iter(&'b mut self) -> Self::Iter {
        <[T]>::iter(self)
    }
}

impl<'a, 'b, T: 'static> GetItem<'b> for RwLockReadGuard<'a, Vec<T>> {
    type Item = &'b T;

    fn get_item(&'b mut self, row: usize) -> Option<Self::Item> {
        self.get(row)
    }
}

impl<'a, 'b, T: 'static> QueryIter<'b> for RwLockWriteGuard<'a, Vec<T>> {
    type Iter = std::slice::IterMut<'b, T>;
    fn iter(&'b mut self) -> Self::Iter {
        <[T]>::iter_mut(self)
    }
}

impl<'a, 'b, T: 'static> GetItem<'b> for RwLockWriteGuard<'a, Vec<T>> {
    type Item = &'b mut T;

    fn get_item(&'b mut self, row: usize) -> Option<Self::Item> {
        self.get_mut(row)
    }
}

pub trait Table<'a> {
    type Item;

    fn get_row(&'a mut self, row: usize) -> Option<Self::Item>;
}

impl<'a, A: GetItem<'a>> Table<'a> for (A,) {
    type Item = (A::Item,);

    fn get_row(&'a mut self, row: usize) -> Option<Self::Item> {
        match self.0.get_item(row) {
            Some(item) => Some((item,)),
            None => None,
        }
    }
}

impl<'a, A: GetItem<'a>, B: GetItem<'a>> Table<'a> for (A, B) {
    type Item = (A::Item, B::Item);
    fn get_row(&'a mut self, row: usize) -> Option<Self::Item> {
        match (self.0.get_item(row), self.1.get_item(row)) {
            (Some(item1), Some(item2)) => Some((item1, item2)),
            _ => None,
        }
    }
}

impl<'a, 'b, A: QueryParameter> QueryIter<'b> for Query<'a, (A,)>
where
    <<A as QueryParameter>::Item as QueryParameterFetch<'a>>::ArchetypeFetchItem: QueryIter<'b>,
{
    type Iter = QueryIterator<
    <<<A as QueryParameter>::Item as QueryParameterFetch<'a>>::ArchetypeFetchItem as QueryIter<
    'b,
>>::Iter
    >;
    fn iter(&'b mut self) -> Self::Iter {
        QueryIterator::new(self.data.iter_mut().map(|a| a.iter()).collect())
    }
}

impl<'a, 'b, A: QueryParameter, B: QueryParameter> QueryIter<'b> for Query<'a, (A, B)>
where
    <<A as QueryParameter>::Item as QueryParameterFetch<'a>>::ArchetypeFetchItem: QueryIter<'b>,
    <<B as QueryParameter>::Item as QueryParameterFetch<'a>>::ArchetypeFetchItem: QueryIter<'b>,
{
    type Iter = QueryIterator<
        Zip<
            <<<A as QueryParameter>::Item as QueryParameterFetch<'a>>::ArchetypeFetchItem as QueryIter<
                'b,
            >>::Iter,
            <<<B as QueryParameter>::Item as QueryParameterFetch<'a>>::ArchetypeFetchItem as QueryIter<
                'b,
            >>::Iter,
        >,
    >;
    fn iter(&'b mut self) -> Self::Iter {
        QueryIterator::new(
            self.data
                .iter_mut()
                .map(|(a, b)| zip(a.iter(), b.iter()))
                .collect(),
        )
    }
}

impl<'a, 'b, A: QueryParameter, B: QueryParameter, C: QueryParameter> QueryIter<'b>
    for Query<'a, (A, B, C)>
where
    <<A as QueryParameter>::Item as QueryParameterFetch<'a>>::ArchetypeFetchItem: QueryIter<'b>,
    <<B as QueryParameter>::Item as QueryParameterFetch<'a>>::ArchetypeFetchItem: QueryIter<'b>,
    <<C as QueryParameter>::Item as QueryParameterFetch<'a>>::ArchetypeFetchItem: QueryIter<'b>,
{
    type Iter = QueryIterator<
        Zip3<
            <<<A as QueryParameter>::Item as QueryParameterFetch<'a>>::ArchetypeFetchItem as QueryIter<
                'b,
            >>::Iter,
            <<<B as QueryParameter>::Item as QueryParameterFetch<'a>>::ArchetypeFetchItem as QueryIter<
                'b,
            >>::Iter,
            <<<C as QueryParameter>::Item as QueryParameterFetch<'a>>::ArchetypeFetchItem as QueryIter<
                'b,
            >>::Iter
        >,
    >;
    fn iter(&'b mut self) -> Self::Iter {
        QueryIterator::new(
            self.data
                .iter_mut()
                .map(|(a, b, c)| Zip3::new(a.iter(), b.iter(), c.iter()))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {

    use crate::{component::Component, world::World};

    use super::QueryIter;

    #[derive(Debug, PartialEq)]
    struct Test1 {
        data: u32,
    }
    impl Component for Test1 {}

    #[derive(Debug)]
    struct Test2 {
        data: u32,
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

        world.spawn((Test1 { data: 3 }, Test2 { data: 3 }));
        world.spawn(Test1 { data: 3 });

        let mut query = world.query::<(&Test1,)>();

        for f in query.iter() {}

        println!("{:?}", query.data);

        assert_eq!(query.data.len(), 2);
    }

    #[test]
    fn tuple_query() {
        let mut world = World::default();

        world.spawn((Test1 { data: 3 }, Test2 { data: 3 }));
        world.spawn(Test1 { data: 3 });

        let mut query = world.query::<(&mut Test1, &Test2)>();

        for (a, b) in query.iter() {
            a.data = 5;
        }

        println!("{:?}", query.data);

        assert_eq!(query.data.len(), 1);
    }

    #[test]
    fn tuple_query_3() {
        let mut world = World::default();

        world.spawn((Test1 { data: 3 }, Test2 { data: 3 }));
        world.spawn(Test1 { data: 3 });
        world.spawn(Test3 { data: 3 });

        let mut query = world.query::<(&mut Test1, &Test2, &Test3)>();

        for (a, b, c) in query.iter() {
            a.data = 5;
        }

        println!("{:?}", query.data);

        assert_eq!(query.data.len(), 1);
    }
}

use std::{
    any::TypeId,
    iter::Zip,
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use itertools::izip;

use crate::{archetype::Archetype, world::World};

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

impl<'a, 'b, T: 'static> QueryIter<'b> for RwLockWriteGuard<'a, Vec<T>> {
    type Iter = std::slice::IterMut<'b, T>;
    fn iter(&'b mut self) -> Self::Iter {
        <[T]>::iter_mut(self)
    }
}

impl<'a, 'b, A: QueryParameter> QueryIter<'b> for Query<'a, (A,)>
where
    <<A as QueryParameter>::Item as QueryParameterFetch<'a>>::ArchetypeFetchItem: QueryIter<'b>,
{
    type Iter = ChainedIterator<
    <<<A as QueryParameter>::Item as QueryParameterFetch<'a>>::ArchetypeFetchItem as QueryIter<
    'b,
>>::Iter
    >;
    fn iter(&'b mut self) -> Self::Iter {
        ChainedIterator::new(self.data.iter_mut().map(|a| a.iter()).collect())
    }
}

impl<'a, 'b, A: QueryParameter, B: QueryParameter> QueryIter<'b> for Query<'a, (A, B)>
where
    <<A as QueryParameter>::Item as QueryParameterFetch<'a>>::ArchetypeFetchItem: QueryIter<'b>,
    <<B as QueryParameter>::Item as QueryParameterFetch<'a>>::ArchetypeFetchItem: QueryIter<'b>,
{
    type Iter = ChainedIterator<
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
        ChainedIterator::new(
            self.data
                .iter_mut()
                .map(|(a, b)| izip!(a.iter(), b.iter()))
                .collect(),
        )
    }
}

#[doc(hidden)]
/// A series of iterators of the same type that are traversed in a row.
pub struct ChainedIterator<I: Iterator> {
    current_iter: Option<I>,
    iterators: Vec<I>,
}

impl<I: Iterator> ChainedIterator<I> {
    #[doc(hidden)]
    pub fn new(mut iterators: Vec<I>) -> Self {
        let current_iter = iterators.pop();
        Self {
            current_iter,
            iterators,
        }
    }
}

impl<I: Iterator> Iterator for ChainedIterator<I> {
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Chain the iterators together.
        // If the end of one iterator is reached go to the next.

        match self.current_iter {
            Some(ref mut iter) => match iter.next() {
                None => {
                    self.current_iter = self.iterators.pop();
                    if let Some(ref mut iter) = self.current_iter {
                        iter.next()
                    } else {
                        None
                    }
                }
                item => item,
            },
            None => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut min = 0;
        let mut max = 0;

        if let Some(current_iter) = &self.current_iter {
            let (i_min, i_max) = current_iter.size_hint();
            min += i_min;
            max += i_max.unwrap();
        }

        for i in self.iterators.iter() {
            let (i_min, i_max) = i.size_hint();
            min += i_min;
            // This function is designed under the assumption that all
            // iterators passed in implement size_hint, which works fine
            // for kudo's purposes.
            max += i_max.unwrap();
        }
        (min, Some(max))
    }
}

#[cfg(test)]
mod tests {
    use std::{iter::Map, ops::Range};

    use itertools::{izip, Zip};

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
}

use std::iter::{zip, Zip};

use crate::World;
use zengine_macro::{query_iter_for_tuple, query_iter_mut_for_tuple};

mod query_fetch;
mod query_iterators;

pub use query_fetch::*;
pub use query_iterators::*;

pub struct QueryRunner<T: QueryParameters> {
    _marker: std::marker::PhantomData<T>,
    query_cache: Option<QueryCache>,
}

impl<T: QueryParameters> QueryRunner<T> {
    pub fn run<'a>(&mut self, world: &'a World) -> Query<'a, T> {
        Query {
            data: T::fetch(world, &mut self.query_cache),
        }
    }
}

impl<T: QueryParameters> Default for QueryRunner<T> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData::default(),
            query_cache: None,
        }
    }
}

pub struct Query<'a, T: QueryParameters> {
    pub data: <T as QueryParameterFetch<'a>>::FetchItem,
}

pub struct QueryCache {
    pub last_archetypes_count: usize,
    matched_archetypes: Vec<(usize, Vec<Option<usize>>)>,
}

query_iter_for_tuple!(14);
query_iter_mut_for_tuple!(14);

#[cfg(test)]
mod tests {

    use crate::{component::Component, query::QueryIterMut, world::World};

    #[derive(Debug, PartialEq)]
    struct Test1 {
        data: u32,
    }
    impl Component for Test1 {}

    #[derive(Debug, PartialEq)]
    struct Test2 {
        _data: u32,
    }
    impl Component for Test2 {}

    #[derive(Debug, PartialEq)]
    struct Test3 {
        data: u32,
    }
    impl Component for Test3 {}

    #[test]
    fn simple_query() {
        let mut world = World::default();

        world.spawn((Test1 { data: 3 }, Test2 { _data: 3 }));
        world.spawn(Test1 { data: 2 });

        let mut query = world.query::<(&Test1,)>();
        let mut query = query.run(&world);

        assert_eq!(query.iter_mut().count(), 2);
    }

    #[test]
    fn tuple_query() {
        let mut world = World::default();

        world.spawn((Test1 { data: 3 }, Test2 { _data: 3 }));
        world.spawn(Test1 { data: 3 });

        let mut query = world.query::<(&Test1, &Test2)>();
        let mut query = query.run(&world);

        assert_eq!(query.iter_mut().count(), 1);
    }

    #[test]
    fn tuple_with_mutable_query() {
        let mut world = World::default();

        world.spawn((Test1 { data: 3 }, Test2 { _data: 3 }));
        world.spawn((Test1 { data: 3 }, Test2 { _data: 2 }));
        world.spawn(Test1 { data: 3 });

        let mut query = world.query::<(&mut Test1, &Test2)>();
        let mut query = query.run(&world);

        assert_eq!(query.iter_mut().count(), 2);

        for (a, _b) in query.iter_mut() {
            a.data = 5;
        }

        for (a, _b) in query.iter_mut() {
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
        let mut query = query.run(&world);
        assert_eq!(query.iter_mut().count(), 2);

        for (a, _b, c) in query.iter_mut() {
            a.data = 5;
            c.data = 7;
        }

        for (a, _b, c) in query.iter_mut() {
            assert_eq!(a.data, 5);
            assert_eq!(c.data, 7);
        }
    }

    #[test]
    fn optional_component_query() {
        let mut world = World::default();

        world.spawn((Test1 { data: 3 }, Test2 { _data: 3 }, Test3 { data: 3 }));
        world.spawn(Test1 { data: 4 });
        world.spawn(Test3 { data: 3 });
        world.spawn((Test1 { data: 5 }, Test2 { _data: 4 }, Test3 { data: 3 }));

        let mut query = world.query::<(&Test1, Option<&Test2>)>();
        let mut query = query.run(&world);
        assert_eq!(query.iter_mut().count(), 3);

        let mut iter = query.iter_mut();
        let data1 = iter.next();
        assert_eq!(data1, Some((&Test1 { data: 4 }, None)));

        let data2 = iter.next();
        assert_eq!(data2, Some((&Test1 { data: 3 }, Some(&Test2 { _data: 3 }))));

        let data3 = iter.next();
        assert_eq!(data3, Some((&Test1 { data: 5 }, Some(&Test2 { _data: 4 }))));
    }
}

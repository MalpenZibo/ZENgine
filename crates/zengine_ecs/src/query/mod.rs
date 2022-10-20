use std::iter::{zip, Zip};

use crate::World;
use zengine_macro::{query_iter_for_tuple, query_iter_mut_for_tuple};

mod query_fetch;
mod query_iterators;

pub use query_fetch::*;
pub use query_iterators::*;

/// Provides access to Entities and components in the world
pub struct QueryRunner<T: QueryParameters> {
    _marker: std::marker::PhantomData<T>,
    query_cache: Option<QueryCache>,
}

impl<T: QueryParameters> QueryRunner<T> {
    /// Runs the query using a reference to the [World]
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

/// Contains result of a [QueryRunner] execution providing access to
/// entities and components in the [World]
///
/// Queries enable iteration over entities and their components.
/// A query matches its parameters against the world to produce a series of results.
/// Each query result is a tuple of components (the same components defined in the query)
/// that belong to the same entity.
///
/// Computational cost of queries is reduced by the fact that they have an internal
/// cache to avoid re-computing matches on each query run.
///
/// # Query as System Parameter
/// A Query can be used as a system parameter. Internally it create a [QueryRunner]
/// and run it on each system execution.
///
/// # Immutable component access
/// The following example defines a query that gives an iterator over
/// `(&ComponentA, &ComponentB)` tuples where ComponentA and ComponentB
/// belong to the same entity
/// ```ignore
/// query: Query<(&ComponentA, &ComponentB)>
/// ```
///
/// # Mutable component access
/// The following example is similar to the previous one, with the exception
/// of `ComponentA` being accessed mutably here
/// ```ignore
/// mut query: Query<(&mut ComponentA, &ComponentB)>
/// ```
///
/// # Add Entity ID to the Query
/// Inserting [Entity](crate::Entity) at any position in the type parameter tuple will give access
/// to the entity ID.
/// ```ignore
/// query: Query<(Entity, &ComponentA, &ComponentB)>
/// ```
///
/// # Optional component access
/// A component can be made optional in a query by wrapping it into an Option.
/// In the following example, the query will iterate over components of both entities that contain
/// `ComponentA` and `ComponentB`, and entities that contain `ComponentA` but not `ComponentB`.
///
/// # Iteration over query result
/// The `iter` and `iter_mut` methods are used to iterate over query result.
/// Refer to the [Iterator API docs](Iterator) for advanced iterator usage.
///
/// ```
/// use zengine_macro::Component;
/// use zengine_ecs::{
///     World,
///     query::{Query, QueryIter, QueryIterMut}
/// };
///
/// #[derive(Component, Debug)]
/// struct ComponentA {}
///
/// #[derive(Component, Debug)]
/// struct ComponentB {}
///
/// fn immutable_query(query: Query<(&ComponentA, &ComponentB)>) {
///     for (a,b) in query.iter() {
///         // a and b are immutable reference to ComponentA and ComponentB
///     }
/// }
///
/// fn mutable_query(mut query: Query<(&mut ComponentA, &ComponentB)>) {
///     for (mut a,b) in query.iter_mut() {
///         // a is a mutable reference to ComponentA
///         // b is an immutable reference to ComponentB
///     }
/// }
/// ```
pub struct Query<'a, T: QueryParameters> {
    pub data: <T as QueryParameterFetch<'a>>::FetchItem,
}

/// Cache query execution information
pub struct QueryCache {
    last_archetypes_count: usize,
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

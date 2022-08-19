use crate::{
    system::{Query, QueryCache, QueryParameters},
    world::World,
};

use super::{SystemParam, SystemParamFetch};

pub struct QueryState<T: QueryParameters> {
    _marker: std::marker::PhantomData<T>,
    query_cache: Option<QueryCache>,
}

impl<T: QueryParameters> Default for QueryState<T> {
    fn default() -> Self {
        QueryState {
            _marker: std::marker::PhantomData::default(),
            query_cache: None,
        }
    }
}

impl<'a, T: QueryParameters> SystemParamFetch<'a> for QueryState<T> {
    type Item = Query<'a, T>;

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        Query {
            data: T::fetch(world, &mut self.query_cache),
        }
    }
}

impl<'a, T: QueryParameters> SystemParam for Query<'a, T> {
    type Fetch = QueryState<T>;
}

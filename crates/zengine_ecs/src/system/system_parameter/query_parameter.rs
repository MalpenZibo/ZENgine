use super::{SystemParam, SystemParamFetch};
use crate::{
    query::{Query, QueryParameters, QueryRunner},
    World,
};

#[doc(hidden)]
pub struct QueryState<T: QueryParameters> {
    query_runner: QueryRunner<T>,
}

impl<T: QueryParameters> Default for QueryState<T> {
    fn default() -> Self {
        Self {
            query_runner: QueryRunner::default(),
        }
    }
}

impl<'a, T: QueryParameters> SystemParamFetch<'a> for QueryState<T> {
    type Item = Query<'a, T>;

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        self.query_runner.run(world)
    }
}

impl<'a, T: QueryParameters> SystemParam for Query<'a, T> {
    type Fetch = QueryState<T>;
}

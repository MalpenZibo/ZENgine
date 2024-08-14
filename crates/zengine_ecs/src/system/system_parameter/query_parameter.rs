use crate::query::{Query, QueryParameters, QueryRunner, ReadOnlyQuery, ReadOnlyQueryParameters, ReadOnlyQueryRunner};

use super::{ReadOnlySystemParam, SystemParam};

impl<'a, T: QueryParameters> SystemParam for Query<'a, T> {
    type State = QueryRunner<T>;
    type Item<'w, 's> = Query<'w, T>;

    fn get<'w, 's>(world: &'w crate::World, state: &'s mut Self::State) -> Self::Item<'w, 's> {
        state.run(world)
    }
}

impl<'a, T: ReadOnlyQueryParameters> SystemParam for ReadOnlyQuery<'a, T> {
    type State = ReadOnlyQueryRunner<T>;
    type Item<'w, 's> = ReadOnlyQuery<'w, T>;

    fn get<'w, 's>(world: &'w crate::World, state: &'s mut Self::State) -> Self::Item<'w, 's> {
        state.run(world)
    }
}

impl<'a, T: QueryParameters> ReadOnlySystemParam for Query<'a, T> {}

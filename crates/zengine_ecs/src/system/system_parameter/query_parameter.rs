use crate::query::{Query, QueryParameters, QueryRunner};

use super::SystemParam;

impl<'a, T: QueryParameters> SystemParam for Query<'a, T> {
    type State = QueryRunner<T>;
    type Item<'w, 's> = Query<'w, T>;

    fn get<'w, 's>(world: &'w crate::World, state: &'s mut Self::State) -> Self::Item<'w, 's> {
        state.run(world)
    }
}

use std::any::Any;

use crate::world::World;

use super::{SystemParam, SystemParamFetch};

pub type Local<'a, T> = &'a mut T;

pub struct LocalState<T: Default> {
    data: T,
}

impl<T: Any + Default> Default for LocalState<T> {
    fn default() -> Self {
        LocalState { data: T::default() }
    }
}

impl<'a, T: Default + 'static> SystemParamFetch<'a> for LocalState<T> {
    type Item = Local<'a, T>;

    fn fetch(&'a mut self, _world: &'a World) -> Self::Item {
        &mut self.data
    }
}

impl<'a, T: Default + 'static> SystemParam for Local<'a, T> {
    type Fetch = LocalState<T>;
}

use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use crate::{
    query::{Query, QueryCache, QueryParameters},
    world::{Resource, World},
};

pub trait SystemParam: Sized {
    type Fetch: for<'a> SystemParamFetch<'a> + Default;
}

pub trait SystemParamFetch<'a> {
    type Item;

    fn init(&mut self, _world: &mut World) {}

    fn fetch(&'a mut self, world: &'a World) -> Self::Item;

    fn apply(&mut self, _world: &mut World) {}
}

pub type SystemParamItem<'a, P> = <<P as SystemParam>::Fetch as SystemParamFetch<'a>>::Item;

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

pub type Res<'a, R> = Option<RwLockReadGuard<'a, R>>;

pub struct ResState<R: Resource> {
    _marker: std::marker::PhantomData<R>,
}

impl<T: Resource> Default for ResState<T> {
    fn default() -> Self {
        ResState {
            _marker: std::marker::PhantomData::default(),
        }
    }
}

impl<'a, R: Resource> SystemParamFetch<'a> for ResState<R> {
    type Item = Res<'a, R>;

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        world.get_resource()
    }
}

impl<'a, R: Resource> SystemParam for Res<'a, R> {
    type Fetch = ResState<R>;
}

pub type ResMut<'a, R> = Option<RwLockWriteGuard<'a, R>>;

pub struct ResMutState<R: Resource> {
    _marker: std::marker::PhantomData<R>,
}

impl<T: Resource> Default for ResMutState<T> {
    fn default() -> Self {
        ResMutState {
            _marker: std::marker::PhantomData::default(),
        }
    }
}

impl<'a, R: Resource> SystemParamFetch<'a> for ResMutState<R> {
    type Item = ResMut<'a, R>;

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        world.get_mut_resource()
    }
}

impl<'a, R: Resource> SystemParam for ResMut<'a, R> {
    type Fetch = ResMutState<R>;
}

pub type Local<'a, T> = &'a mut T;

pub struct LocalState<T> {
    data: T,
}

impl<T: Default + 'static> Default for LocalState<T> {
    fn default() -> Self {
        LocalState { data: T::default() }
    }
}

impl<'a, T: 'static> SystemParamFetch<'a> for LocalState<T> {
    type Item = Local<'a, T>;

    fn fetch(&'a mut self, _world: &'a World) -> Self::Item {
        &mut self.data
    }
}

impl<'a, T: Default + 'static> SystemParam for Local<'a, T> {
    type Fetch = LocalState<T>;
}

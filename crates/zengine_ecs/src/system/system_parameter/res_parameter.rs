use std::{
    cell::{Ref, RefMut},
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use crate::{Resource, UnsendableResource, World};

use super::{SystemParam, SystemParamFetch};

pub type Res<'a, R> = RwLockReadGuard<'a, R>;

pub struct ResState<R: Resource + Default> {
    _marker: std::marker::PhantomData<R>,
}

impl<T: Resource + Default> Default for ResState<T> {
    fn default() -> Self {
        ResState {
            _marker: std::marker::PhantomData::default(),
        }
    }
}

impl<'a, R: Resource + Default> SystemParamFetch<'a> for ResState<R> {
    type Item = Res<'a, R>;

    fn init(&mut self, world: &mut World) {
        if world.get_resource::<R>().is_none() {
            world.create_resource(R::default())
        }
    }

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        world.get_resource().unwrap()
    }
}

impl<'a, R: Resource + Default> SystemParam for Res<'a, R> {
    type Fetch = ResState<R>;
}

pub type ResMut<'a, R> = RwLockWriteGuard<'a, R>;

pub struct ResMutState<R: Resource + Default> {
    _marker: std::marker::PhantomData<R>,
}

impl<T: Resource + Default> Default for ResMutState<T> {
    fn default() -> Self {
        ResMutState {
            _marker: std::marker::PhantomData::default(),
        }
    }
}

impl<'a, R: Resource + Default> SystemParamFetch<'a> for ResMutState<R> {
    type Item = ResMut<'a, R>;

    fn init(&mut self, world: &mut World) {
        if world.get_resource::<R>().is_none() {
            world.create_resource::<R>(R::default())
        }
    }

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        world.get_mut_resource().unwrap()
    }
}

impl<'a, R: Resource + Default> SystemParam for ResMut<'a, R> {
    type Fetch = ResMutState<R>;
}

pub type OptionalRes<'a, R> = Option<RwLockReadGuard<'a, R>>;

pub struct OptionalResState<R: Resource> {
    _marker: std::marker::PhantomData<R>,
}

impl<T: Resource> Default for OptionalResState<T> {
    fn default() -> Self {
        OptionalResState {
            _marker: std::marker::PhantomData::default(),
        }
    }
}

impl<'a, R: Resource> SystemParamFetch<'a> for OptionalResState<R> {
    type Item = OptionalRes<'a, R>;

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        world.get_resource()
    }
}

impl<'a, R: Resource> SystemParam for OptionalRes<'a, R> {
    type Fetch = OptionalResState<R>;
}

pub type OptionalResMut<'a, R> = Option<RwLockWriteGuard<'a, R>>;

pub struct OptionalResMutState<R: Resource> {
    _marker: std::marker::PhantomData<R>,
}

impl<T: Resource> Default for OptionalResMutState<T> {
    fn default() -> Self {
        OptionalResMutState {
            _marker: std::marker::PhantomData::default(),
        }
    }
}

impl<'a, R: Resource> SystemParamFetch<'a> for OptionalResMutState<R> {
    type Item = OptionalResMut<'a, R>;

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        world.get_mut_resource()
    }
}

impl<'a, R: Resource> SystemParam for OptionalResMut<'a, R> {
    type Fetch = OptionalResMutState<R>;
}

pub type UnsendableRes<'a, R> = Ref<'a, R>;

pub struct UnsendableResState<R: UnsendableResource + Default> {
    _marker: std::marker::PhantomData<R>,
}

impl<T: UnsendableResource + Default> Default for UnsendableResState<T> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData::default(),
        }
    }
}

impl<'a, R: UnsendableResource + Default> SystemParamFetch<'a> for UnsendableResState<R> {
    type Item = UnsendableRes<'a, R>;

    fn init(&mut self, world: &mut World) {
        if world.get_unsendable_resource::<R>().is_none() {
            world.create_unsendable_resource(R::default());
        }
    }

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        world.get_unsendable_resource().unwrap()
    }
}

impl<'a, R: UnsendableResource + Default> SystemParam for UnsendableRes<'a, R> {
    type Fetch = UnsendableResState<R>;
}

pub type UnsendableResMut<'a, R> = RefMut<'a, R>;

pub struct UnsendableResMutState<R: UnsendableResource + Default> {
    _marker: std::marker::PhantomData<R>,
}

impl<T: UnsendableResource + Default> Default for UnsendableResMutState<T> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData::default(),
        }
    }
}

impl<'a, R: UnsendableResource + Default> SystemParamFetch<'a> for UnsendableResMutState<R> {
    type Item = UnsendableResMut<'a, R>;

    fn init(&mut self, world: &mut World) {
        if world.get_unsendable_resource::<R>().is_none() {
            world.create_unsendable_resource(R::default())
        }
    }

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        world.get_mut_unsendable_resource().unwrap()
    }
}

impl<'a, R: UnsendableResource + Default> SystemParam for UnsendableResMut<'a, R> {
    type Fetch = UnsendableResMutState<R>;
}

pub type OptionalUnsendableRes<'a, R> = Option<Ref<'a, R>>;

pub struct OptionalUnsendableResState<R: UnsendableResource> {
    _marker: std::marker::PhantomData<R>,
}

impl<T: UnsendableResource> Default for OptionalUnsendableResState<T> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData::default(),
        }
    }
}

impl<'a, R: UnsendableResource> SystemParamFetch<'a> for OptionalUnsendableResState<R> {
    type Item = OptionalUnsendableRes<'a, R>;

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        world.get_unsendable_resource()
    }
}

impl<'a, R: UnsendableResource> SystemParam for OptionalUnsendableRes<'a, R> {
    type Fetch = OptionalUnsendableResState<R>;
}

pub type OptionalUnsendableResMut<'a, R> = Option<RefMut<'a, R>>;

pub struct OptionalUnsendableResMutState<R: UnsendableResource> {
    _marker: std::marker::PhantomData<R>,
}

impl<T: UnsendableResource> Default for OptionalUnsendableResMutState<T> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData::default(),
        }
    }
}

impl<'a, R: UnsendableResource> SystemParamFetch<'a> for OptionalUnsendableResMutState<R> {
    type Item = OptionalUnsendableResMut<'a, R>;

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        world.get_mut_unsendable_resource()
    }
}

impl<'a, R: UnsendableResource> SystemParam for OptionalUnsendableResMut<'a, R> {
    type Fetch = OptionalUnsendableResMutState<R>;
}

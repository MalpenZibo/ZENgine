use std::{
    cell::{Ref, RefMut}, ops::{Deref, DerefMut}, sync::{RwLockReadGuard, RwLockWriteGuard}
};

use crate::{Resource, UnsendableResource, World};

use super::SystemParam;

pub struct Res<'a, R: Resource>(RwLockReadGuard<'a, R>);
unsafe impl<'a, T: Resource> Send for Res<'a, T> {}

impl<R: Resource> Deref for Res<'_, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: Resource + Default> SystemParam for Res<'a, T> {
    type State = ();
    type Item<'w, 's> = Res<'w, T>;

    fn init(world: &mut World, _state: &mut Self::State) {
        if world.get_resource::<T>().is_none() {
            world.create_resource(T::default())
        }
    }

    fn get<'w, 's>(world: &'w World, _state: &'s mut Self::State) -> Self::Item<'w, 's> {
        Res(world.get_resource::<T>().unwrap())
    }
}

impl<'a, T: Resource > SystemParam for Option<Res<'a, T>> {
    type State = ();
    type Item<'w, 's> = Option<Res<'w, T>>;

    fn get<'w, 's>(world: &'w World, _state: &'s mut Self::State) -> Self::Item<'w, 's> {
        world.get_resource::<T>().map(Res)
    }
}

pub struct ResMut<'a, R: Resource>(RwLockWriteGuard<'a, R>);
unsafe impl<'a, T: Resource> Send for ResMut<'a, T> {}

impl<R: Resource> Deref for ResMut<'_, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<R: Resource> DerefMut for ResMut<'_, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T: Resource + Default> SystemParam for ResMut<'a, T> {
    type State = ();
    type Item<'w, 's> = ResMut<'w, T>;

    fn init(world: &mut World, _state: &mut Self::State) {
        if world.get_resource::<T>().is_none() {
            world.create_resource(T::default())
        }
    }

    fn get<'w, 's>(world: &'w World, _state: &'s mut Self::State) -> Self::Item<'w, 's> {
        ResMut(world.get_mut_resource::<T>().unwrap())
    }
}

impl<'a, T: Resource> SystemParam for Option<ResMut<'a, T>> {
    type State = ();
    type Item<'w, 's> = Option<ResMut<'w, T>>;

    fn get<'w, 's>(world: &'w World, _state: &'s mut Self::State) -> Self::Item<'w, 's> {
        world.get_mut_resource::<T>().map(ResMut)
    }
}

pub struct UnsendableRes<'a, R: UnsendableResource>(Ref<'a, R>);
unsafe impl<'a, T: UnsendableResource> Sync for UnsendableRes<'a, T> {}

impl<R: UnsendableResource> Deref for UnsendableRes<'_, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: UnsendableResource + Default> SystemParam for UnsendableRes<'a, T> {
    type State = ();
    type Item<'w, 's> = UnsendableRes<'w, T>;

    fn init(world: &mut World, _state: &mut Self::State) {
        if world.get_unsendable_resource::<T>().is_none() {
            world.create_unsendable_resource(T::default())
        }
    }

    fn get<'w, 's>(world: &'w World, _state: &'s mut Self::State) -> Self::Item<'w, 's> {
        UnsendableRes(world.get_unsendable_resource::<T>().unwrap())
    }
}

impl<'a, T: UnsendableResource > SystemParam for Option<UnsendableRes<'a, T>> {
    type State = ();
    type Item<'w, 's> = Option<UnsendableRes<'w, T>>;

    fn get<'w, 's>(world: &'w World, _state: &'s mut Self::State) -> Self::Item<'w, 's> {
        world.get_unsendable_resource::<T>().map(UnsendableRes)
    }
}

pub struct UnsendableResMut<'a, R: UnsendableResource>(RefMut<'a, R>);
unsafe impl<'a, T: UnsendableResource> Sync for UnsendableResMut<'a, T> {}

impl<R: UnsendableResource> Deref for UnsendableResMut<'_, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<R: UnsendableResource> DerefMut for UnsendableResMut<'_, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T: UnsendableResource + Default> SystemParam for UnsendableResMut<'a, T> {
    type State = ();
    type Item<'w, 's> = UnsendableResMut<'w, T>;

    fn init(world: &mut World, _state: &mut Self::State) {
        if world.get_unsendable_resource::<T>().is_none() {
            world.create_unsendable_resource(T::default())
        }
    }

    fn get<'w, 's>(world: &'w World, _state: &'s mut Self::State) -> Self::Item<'w, 's> {
        UnsendableResMut(world.get_mut_unsendable_resource::<T>().unwrap())
    }
}

impl<'a, T: UnsendableResource> SystemParam for Option<UnsendableResMut<'a, T>> {
    type State = ();
    type Item<'w, 's> = Option<UnsendableResMut<'w, T>>;

    fn get<'w, 's>(world: &'w World, _state: &'s mut Self::State) -> Self::Item<'w, 's> {
        world.get_mut_unsendable_resource::<T>().map(UnsendableResMut)
    }
}

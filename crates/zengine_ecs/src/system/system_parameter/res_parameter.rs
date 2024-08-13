use std::{
    cell::{Ref, RefMut}, ops::{Deref, DerefMut}, sync::{RwLockReadGuard, RwLockWriteGuard}
};

use crate::{Resource, UnsendableResource, World};

use super::SystemParam;

/// Shared borrow of a resource that implements also the [Default] trait
///
/// If you need a resource that doesn't implement Default, use `Option<Res<T>>` instead
/// If you need a unique mutable borrow, use [ResMut] instead.
///
/// # Example
/// ```
/// use zengine_macro::Resource;
/// use zengine_ecs::system::Res;
///
/// #[derive(Resource, Debug)]
/// struct ResourceA {}
///
/// fn my_system(res: Res<ResourceA>) {
///     println!("ResourceA {:?}", res);
/// }
///
/// fn my_system_option(res: Option<Res<ResourceA>>) {
///     if let Some(res) = res {
///         println!("ResourceA {:?}", res);
///     }
/// }
/// ```
#[derive(Debug)]
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

/// Unique mutable borrow of a resource that implements also the [Default] trait
///
/// If you need a resource that doesn't implement Default, use `Option<ResMut<T>>` instead
/// If you need a shared borrow, use [ResMut] instead.
///
/// # Example
/// ```
/// use zengine_macro::Resource;
/// use zengine_ecs::system::ResMut;
///
/// #[derive(Resource, Default, Debug)]
/// struct ResourceA {
///     data: u32
/// }
///
/// #[derive(Resource, Debug)]
/// struct ResourceB {
///     data: u32
/// }
///
/// fn my_system(mut res: ResMut<ResourceA>) {
///     res.data = 6;
/// }
///
/// fn my_system_mut(res: Option<ResMut<ResourceB>>) {
///     if let Some(mut res) = res {
///         res.data = 6;
///     }
/// }
/// ```
#[derive(Debug)]
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

/// Shared borrow of an unsendable resource that implements also the [Default] trait
///
/// If you need an unsendable resource that doesn't implement Default, use `Option<UnsendableRes<T>>` instead
/// If you need a unique mutable borrow, use [UnsendableResMut] instead.
///
/// # Example
/// ```
/// use zengine_macro::Resource;
/// use zengine_ecs::system::UnsendableRes;
///
/// #[derive(Resource, Default, Debug)]
/// struct ResourceA {}
///
/// #[derive(Resource, Debug)]
/// struct ResourceB {}
///
/// fn my_system(res: UnsendableRes<ResourceA>) {
///     println!("Unsendable ResourceA {:?}", res);
/// }
///
/// fn my_system_mut(res: Option<UnsendableRes<ResourceB>>) {
///     if let Some(res) = res {
///         println!("Unsendable ResourceA {:?}", res);
///     }
/// }
/// ```
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

/// Unique mutable borrow of an unsendable resource that implements also the [Default] trait
///
/// If you need an unsendable resource that doesn't implement Default, use `Option<UnsendableResMut<T>>` instead
/// If you need a shared borrow, use [UnsendableResMut] instead.
///
/// # Example
/// ```
/// use zengine_macro::Resource;
/// use zengine_ecs::system::UnsendableResMut;
///
/// #[derive(Resource, Default, Debug)]
/// struct ResourceA {
///     data: u32
/// }
///
/// #[derive(Resource, Debug)]
/// struct ResourceB {
///     data: u32
/// }
///
/// fn my_system(mut res: UnsendableResMut<ResourceA>) {
///     res.data = 6;
/// }
///
/// fn my_system_mut(res: Option<UnsendableResMut<ResourceB>>) {
///     if let Some(mut res) = res {
///         res.data = 6;
///     }
/// }
/// ```
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

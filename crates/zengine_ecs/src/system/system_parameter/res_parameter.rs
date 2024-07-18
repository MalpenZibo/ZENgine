use std::{
    cell::{Ref, RefMut},
    marker::PhantomData,
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use crate::{Resource, UnsendableResource, World};

use super::{SystemParam, SystemParamFetch};

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
pub type Res<'a, R> = RwLockReadGuard<'a, R>;

#[doc(hidden)]
pub struct ResState<R: Resource + Default> {
    _marker: std::marker::PhantomData<R>,
}

impl<T: Resource + Default> Default for ResState<T> {
    fn default() -> Self {
        ResState {
            _marker: PhantomData,
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
pub type ResMut<'a, R> = RwLockWriteGuard<'a, R>;

#[doc(hidden)]
pub struct ResMutState<R: Resource + Default> {
    _marker: std::marker::PhantomData<R>,
}

impl<T: Resource + Default> Default for ResMutState<T> {
    fn default() -> Self {
        ResMutState {
            _marker: PhantomData,
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

#[doc(hidden)]
pub struct OptionalResState<R: Resource> {
    _marker: std::marker::PhantomData<R>,
}

impl<T: Resource> Default for OptionalResState<T> {
    fn default() -> Self {
        OptionalResState {
            _marker: PhantomData,
        }
    }
}

impl<'a, R: Resource> SystemParamFetch<'a> for OptionalResState<R> {
    type Item = Option<Res<'a, R>>;

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        world.get_resource()
    }
}

impl<'a, R: Resource> SystemParam for Option<Res<'a, R>> {
    type Fetch = OptionalResState<R>;
}

#[doc(hidden)]
pub struct OptionalResMutState<R: Resource> {
    _marker: std::marker::PhantomData<R>,
}

impl<T: Resource> Default for OptionalResMutState<T> {
    fn default() -> Self {
        OptionalResMutState {
            _marker: PhantomData,
        }
    }
}

impl<'a, R: Resource> SystemParamFetch<'a> for OptionalResMutState<R> {
    type Item = Option<ResMut<'a, R>>;

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        world.get_mut_resource()
    }
}

impl<'a, R: Resource> SystemParam for Option<ResMut<'a, R>> {
    type Fetch = OptionalResMutState<R>;
}

/// Shared borrow of an unsendable resource that implements also the [Default] trait
///
/// If you need an unsendable resource that doesn't implement Default, use `Option<UnsendableRes<T>>` instead
/// If you need a unique mutable borrow, use [UnsendableResMut] instead.
///
/// # Example
/// ```
/// use zengine_macro::UnsendableResource;
/// use zengine_ecs::system::UnsendableRes;
///
/// #[derive(UnsendableResource, Default, Debug)]
/// struct ResourceA {}
///
/// #[derive(UnsendableResource, Debug)]
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
pub type UnsendableRes<'a, R> = Ref<'a, R>;

#[doc(hidden)]
pub struct UnsendableResState<R: UnsendableResource + Default> {
    _marker: std::marker::PhantomData<R>,
}

impl<T: UnsendableResource + Default> Default for UnsendableResState<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
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

/// Unique mutable borrow of an unsendable resource that implements also the [Default] trait
///
/// If you need an unsendable resource that doesn't implement Default, use `Option<UnsendableResMut<T>>` instead
/// If you need a shared borrow, use [UnsendableResMut] instead.
///
/// # Example
/// ```
/// use zengine_macro::UnsendableResource;
/// use zengine_ecs::system::UnsendableResMut;
///
/// #[derive(UnsendableResource, Default, Debug)]
/// struct ResourceA {
///     data: u32
/// }
///
/// #[derive(UnsendableResource, Debug)]
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
pub type UnsendableResMut<'a, R> = RefMut<'a, R>;

#[doc(hidden)]
pub struct UnsendableResMutState<R: UnsendableResource + Default> {
    _marker: std::marker::PhantomData<R>,
}

impl<T: UnsendableResource + Default> Default for UnsendableResMutState<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
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

#[doc(hidden)]
pub struct OptionalUnsendableResState<R: UnsendableResource> {
    _marker: std::marker::PhantomData<R>,
}

impl<T: UnsendableResource> Default for OptionalUnsendableResState<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<'a, R: UnsendableResource> SystemParamFetch<'a> for OptionalUnsendableResState<R> {
    type Item = Option<UnsendableRes<'a, R>>;

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        world.get_unsendable_resource()
    }
}

impl<'a, R: UnsendableResource> SystemParam for Option<UnsendableRes<'a, R>> {
    type Fetch = OptionalUnsendableResState<R>;
}

#[doc(hidden)]
pub struct OptionalUnsendableResMutState<R: UnsendableResource> {
    _marker: std::marker::PhantomData<R>,
}

impl<T: UnsendableResource> Default for OptionalUnsendableResMutState<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<'a, R: UnsendableResource> SystemParamFetch<'a> for OptionalUnsendableResMutState<R> {
    type Item = Option<UnsendableResMut<'a, R>>;

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        world.get_mut_unsendable_resource()
    }
}

impl<'a, R: UnsendableResource> SystemParam for Option<UnsendableResMut<'a, R>> {
    type Fetch = OptionalUnsendableResMutState<R>;
}

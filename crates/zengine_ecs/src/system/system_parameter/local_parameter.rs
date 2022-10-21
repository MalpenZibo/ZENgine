use super::{SystemParam, SystemParamFetch};
use crate::world::World;
use std::any::Any;

/// A mutable system local resource parameter.
///
/// A local may only be accessed by the system itself and is therefore not visible to other systems.
/// If two or more systems specify the same local type each will have their own unique local.
///
/// A Local data type must implement [Resource](crate::Resource) and [Default] trait
///
/// /// # Example
/// ```
/// use zengine_macro::Resource;
/// use zengine_ecs::system::Local;
///
/// #[derive(Resource, Debug)]
/// struct Data {
///     data: u32
/// }
///
/// fn my_system(local: Local<Data>) {
///     local.data = 6;
///     println!("Local resource {:?}", local);
/// }
/// ```
pub type Local<'a, T> = &'a mut T;

#[doc(hidden)]
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

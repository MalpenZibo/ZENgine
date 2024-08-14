//! Tools for controlling behavior in an ECS application
//!
//! Systems define how an ECS application behaves.
//! A system is added to a `Stage` to be able to run.
//! A function that use only system parameters can be converted into a system.
//! Usually this conversion is done automatically.
//!
//! System functions can query and mutate the ZENgine state using its parameters.
//! Only types that implement [SystemParam]
//! can be used, automatically fetching data from the World.
//!
//! # Example
//! ```
//! use zengine_macro::{Component, Resource};
//! use zengine_ecs::{
//!     Entity,
//!     system::ResMut,
//!     query::{Query, QueryIterMut}
//! };
//!
//! #[derive(Component, Debug)]
//! struct HeathPoints(u32);
//!
//! #[derive(Component, Debug)]
//! struct Life(u32);
//!
//! #[derive(Resource, Default, Debug)]
//! struct DeathEntities(Vec<Entity>);
//!
//! fn custom_system(
//!     mut query: Query<(Entity, &HeathPoints, &mut Life)>,
//!     mut death: ResMut<DeathEntities>,
//! ) {
//!     for (entity, hp, life) in query.iter_mut() {
//!         if hp.0 == 0 {
//!             life.0 -= 1;
//!             if life.0 == 0 {
//!                 death.0.push(*entity);
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! # System ordering
//! Systems inside a Stage are executed sequentially based on the insertion order
//!
//! # System Parameters
//! Following is the complete list of accepted types as system parameters:
//! - [Query](crate::query::Query) to query over entities and components
//! - [Res] and `Option<Res>` to get immutable access to a resource (or an optional resource)
//! - [ResMut] and `Option<ResMut>` to get mutable access to a resource (or an optional resource)
//! - [UnsendableRes] and `Option<UnsendableRes>` to get immutable access to
//! an unsendable resource (or an optional unsendable resource)
//! - [UnsendableResMut] and `Option<UnsendableResMut>` to get mutable access to
//! an unsendable resource (or an optional unsendable resource)
//! - [Event] to get access to an event without subscribing
//! - [EventStream] to get access to an event with a subscription
//! - [EventPublisher] to publish an event
//! - [Commands] to send command to the [World]

use crate::World;
use zengine_macro::all_tuples;

mod system_parameter;
pub use system_parameter::*;

/// System trait
pub trait System: Send + Sync {
    fn init(&mut self, world: &mut World);

    fn run(&mut self, world: &World);

    fn apply(&mut self, world: &mut World);
}

/// Conversion trait to turn something into a [BoxedSystem]
pub trait IntoSystem<Marker> {
    fn into_system(self) -> BoxedSystem;
}

pub trait IntoReadOnlySystem<Marker> {
    fn into_system(self) -> BoxedReadOnlySystem;
}

/// A trait implemented for all functions that can be used as a [System]
pub trait SystemFunction<Marker>: Send + Sync + 'static {
    type Param: SystemParam;

    fn run_function(&mut self, data: <Self::Param as SystemParam>::Item<'_, '_>);
}

pub trait ReadOnlySystemFunction<Marker>: Send + Sync + 'static {
    type Param: ReadOnlySystemParam;

    fn run_function(&mut self, data: <Self::Param as SystemParam>::Item<'_, '_>);
}

impl<Marker: 'static, T> IntoSystem<Marker> for T
where
    T: SystemFunction<Marker> + Send + Sync,
{
    fn into_system(self) -> BoxedSystem {
        Box::new(SystemFunctionData {
            function: self,
            param_state: Default::default(),
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<Marker: 'static, T> IntoReadOnlySystem<Marker> for T
where
    T: ReadOnlySystemFunction<Marker> + Send + Sync,
{
    fn into_system(self) -> BoxedReadOnlySystem {
        Box::new(ReadOnlySystemFunctionData {
            function: self,
            param_state: Default::default(),
            _phantom: std::marker::PhantomData,
        })
    }
}

/// Wraps a function that implements the [SystemFunction] trait
pub struct SystemFunctionData<Marker, F: SystemFunction<Marker>> {
    function: F,
    param_state: <F::Param as SystemParam>::State,
    _phantom: std::marker::PhantomData<fn() -> Marker>,
}

pub struct ReadOnlySystemFunctionData<Marker, F: ReadOnlySystemFunction<Marker>> {
    function: F,
    param_state: <F::Param as SystemParam>::State,
    _phantom: std::marker::PhantomData<fn() -> Marker>,
}

pub type BoxedSystem = Box<dyn System>;

pub type BoxedReadOnlySystem = Box<dyn System>;

impl<Marker, F: SystemFunction<Marker>> System for SystemFunctionData<Marker, F> {
    fn init(&mut self, world: &mut World) {
        <F::Param as SystemParam>::init(world, &mut self.param_state);
    }

    fn run(&mut self, world: &World) {
        let data = <F::Param as SystemParam>::get(world, &mut self.param_state);
        self.function.run_function(data);
    }

    fn apply(&mut self, world: &mut World) {
        <F::Param as SystemParam>::apply(world, &mut self.param_state);
    }
}

impl<Marker, F: ReadOnlySystemFunction<Marker>> System for ReadOnlySystemFunctionData<Marker, F> {
    fn init(&mut self, world: &mut World) {
        <F::Param as SystemParam>::init(world, &mut self.param_state);
    }

    fn run(&mut self, world: &World) {
        let data = <F::Param as SystemParam>::get(world, &mut self.param_state);
        self.function.run_function(data);
    }

    fn apply(&mut self, world: &mut World) {
        <F::Param as SystemParam>::apply(world, &mut self.param_state);
    }
}

macro_rules! impl_system_function {
    ($($param: ident),*) => {
        #[allow(non_snake_case)]
        impl<$($param: SystemParam + 'static),*> SystemParam for ($($param,)*) {
            type State = ($($param::State,)*);
            type Item<'w, 's> = ($($param::Item<'w, 's>,)*);

            fn init(_world: &mut World, state: &mut Self::State) {
                let ($($param,)*) = state;
                $($param::init(_world, $param);)*
            }

            fn get<'w, 's>(_world: &'w World, state: &'s mut Self::State) -> Self::Item<'w, 's> {
                let ($($param,)*) = state;
                #[allow(clippy::unused_unit)]
                ($($param::get(_world, $param),)*)
            }

            fn apply(_world: &mut World, state: &mut Self::State) {
                let ($($param,)*) = state;
                $($param::apply(_world, $param);)*
            }
        }

        #[allow(non_snake_case)]
        impl<F: Send + Sync + 'static, $($param: SystemParam + 'static),*> SystemFunction<fn($($param,)*)> for F
        where
            for <'a> &'a mut F: FnMut($($param),*) + FnMut($($param::Item<'_, '_>),*)
        {
            type Param = ($($param,)*);

            fn run_function(&mut self, data: <Self::Param as SystemParam>::Item<'_, '_>) {
                // Yes, this is strange, but `rustc` fails to compile this impl
                // without using this function. It fails to recognize that `func`
                // is a function, potentially because of the multiple impls of `FnMut`
                #[allow(clippy::too_many_arguments)]
                fn call_inner<$($param,)*>(
                    mut f: impl FnMut($($param,)*),
                    $($param: $param,)*
                ){
                    f($($param,)*)
                }

                let ($($param,)*) = data;
                call_inner(self, $($param),*);
            }
        }
    }
}
all_tuples!(impl_system_function, 0, 12, F);

macro_rules! impl_readonly_system_function {
    ($($param: ident),*) => {
        #[allow(non_snake_case)]
        impl<$($param: ReadOnlySystemParam + 'static),*> ReadOnlySystemParam for ($($param,)*) {}

        #[allow(non_snake_case)]
        impl<F: Send + Sync + 'static, $($param: ReadOnlySystemParam + 'static),*> ReadOnlySystemFunction<fn($($param,)*)> for F
        where
            for <'a> &'a mut F: FnMut($($param),*) + FnMut($($param::Item<'_, '_>),*)
        {
            type Param = ($($param,)*);

            fn run_function(&mut self, data: <Self::Param as SystemParam>::Item<'_, '_>) {
                // Yes, this is strange, but `rustc` fails to compile this impl
                // without using this function. It fails to recognize that `func`
                // is a function, potentially because of the multiple impls of `FnMut`
                #[allow(clippy::too_many_arguments)]
                fn call_inner<$($param,)*>(
                    mut f: impl FnMut($($param,)*),
                    $($param: $param,)*
                ){
                    f($($param,)*)
                }

                let ($($param,)*) = data;
                call_inner(self, $($param),*);
            }
        }
    }
}
all_tuples!(impl_readonly_system_function, 0, 12, F);

#[cfg(test)]
mod tests {
    use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

    use crate::Resource;

    use super::{BoxedSystem, EventStream, IntoSystem, Res, ResMut};

    #[derive(Debug, Default)]
    struct T {
        a: i32,
    }
    impl Resource for T {}

    #[derive(Debug, Default)]
    struct W {
        a: i32,
    }
    impl Resource for W {}

    fn system0() {}

    fn system1(a: Res<T>) {
        println!("{:?}", a.a);
    }

    fn system2(a: Res<T>, b: ResMut<W>, _c: EventStream<u32>) {
        println!("{:?}", a.a);
        println!("{:?}", b.a);
    }

    fn system3() -> impl FnMut(Res<T>, ResMut<W>, EventStream<u32>) {
        |a: Res<T>, b: ResMut<W>, _c: EventStream<u32>| {
            println!("{:?}", a.a);
            println!("{:?}", b.a);
        }
    }

    #[test]
    fn test_system() {
        let sys0: BoxedSystem = IntoSystem::into_system(system0);
        let sys1: BoxedSystem = IntoSystem::into_system(system1);
        let sys2: BoxedSystem = IntoSystem::into_system(system2);
        let sys3: BoxedSystem = IntoSystem::into_system(system3());
        let mut systems: Vec<BoxedSystem> = vec![sys0, sys1, sys2, sys3];

        let world = Default::default();

        systems.par_iter_mut().for_each(|s| {
            s.run(&world);
        });
    }
}

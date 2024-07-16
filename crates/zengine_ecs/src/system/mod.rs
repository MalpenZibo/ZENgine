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
//! - [Local] to get access to data owned by the system

use std::marker::PhantomData;

use zengine_macro::all_tuples;

use crate::world::World;

mod system_parameter;
pub use system_parameter::*;

/// A trait implemented for all functions that can be used as a [System]
pub trait SystemFunction<P: SystemParam> {
    fn run_function(&self, parameter: SystemParamItem<P>);
}

/// Conversion trait to turn something into a [System]
pub trait IntoSystem<P: SystemParam> {
    type System: SystemFunction<P>;

    fn into_system(self) -> SystemWrapper<Self::System, P>;
}

impl<Param: SystemParam, F> IntoSystem<Param> for F
where
    F: SystemFunction<Param>,
{
    type System = F;
    fn into_system(self) -> SystemWrapper<Self::System, Param> {
        SystemWrapper {
            _marker: PhantomData,
            function: self,
            param_state: Param::Fetch::default(),
        }
    }
}

/// Wraps a function that implements the [SystemFunction] trait
pub struct SystemWrapper<F: SystemFunction<P>, P: SystemParam> {
    _marker: std::marker::PhantomData<P>,
    function: F,
    param_state: P::Fetch,
}

/// System trait
pub trait System {
    fn init(&mut self, world: &mut World);

    fn run(&mut self, world: &World);

    fn apply(&mut self, world: &mut World);
}

impl<F: SystemFunction<P>, P: SystemParam> System for SystemWrapper<F, P> {
    fn init(&mut self, world: &mut World) {
        self.param_state.init(world);
    }

    fn run(&mut self, world: &World) {
        let data: <<P as SystemParam>::Fetch as SystemParamFetch>::Item =
            <P as SystemParam>::Fetch::fetch(&mut self.param_state, world);
        self.function.run_function(data);
    }

    fn apply(&mut self, world: &mut World) {
        self.param_state.apply(world);
    }
}

macro_rules! impl_system_function {
    () => {
        #[allow(non_snake_case)]
        impl<'a> SystemParamFetch<'a> for () {
            type Item = ();

            fn fetch(&mut self, _world: &'a World) -> Self::Item {}
        }

        impl SystemParam for () {
            type Fetch = ();
        }

        #[allow(non_snake_case)]
        impl<Sys> SystemFunction<()> for Sys
        where
            for<'a> &'a Sys: Fn(),
        {
            fn run_function(&self, _parameter: SystemParamItem<()>) {
                #[allow(clippy::too_many_arguments)]
                fn call_inner(f: impl Fn()) {
                    f()
                }

                call_inner(self)
            }
        }
    };
    ($($param: ident),+) => {
        #[allow(non_snake_case)]
        impl<'a, $($param: SystemParamFetch<'a>),*> SystemParamFetch<'a> for ($($param,)*) {
            type Item = ($($param::Item,)*);

            fn init(&mut self, world: &mut World) {
                let ($($param,)*) = self;

                ($($param::init($param, world),)*);
            }

            fn fetch(&'a mut self, world: &'a World) -> Self::Item {
                let ($($param,)*) = self;

                ($($param::fetch($param, world),)*)
            }

            fn apply(&mut self, world: &mut World) {
                let ($($param,)*) = self;

                ($($param::apply($param, world),)*);
            }
        }

        impl<$($param: SystemParam),*> SystemParam for ($($param,)*) {
            type Fetch = ($($param::Fetch,)*);
        }

        #[allow(non_snake_case)]
        impl<$($param: SystemParam),*, Sys> SystemFunction<($($param,)*)> for Sys
        where
            for<'a> &'a Sys: Fn( $($param),*)
                + Fn(
                    $(<<$param as SystemParam>::Fetch as SystemParamFetch>::Item,)*
                ),
        {
            fn run_function(&self, parameter: SystemParamItem<($($param,)*)>) {
                #[allow(clippy::too_many_arguments)]
                fn call_inner<$($param),*>(f: impl Fn($($param,)*), $($param: $param,)*) {
                    f($($param,)*)
                }

                let ($($param,)*) = parameter;
                call_inner(self, $($param),*)
            }
        }
    }
}
all_tuples!(impl_system_function, 0, 12, F);

#[cfg(test)]
mod tests {

    use crate::{query::Query, world::World, Component, Resource};

    use super::{
        system_parameter::{Local, Res},
        IntoSystem, System, SystemParam,
    };

    #[derive(Default)]
    struct Executor {
        world: World,
        systems: Vec<Box<dyn System>>,
    }

    impl Executor {
        fn add_system<Params: SystemParam + 'static>(
            mut self,
            system: impl IntoSystem<Params> + 'static,
        ) -> Self {
            self.systems.push(Box::new(system.into_system()));

            self
        }
    }

    impl Executor {
        pub fn run(mut self) {
            for s in self.systems.iter_mut() {
                s.init(&mut self.world);
            }

            for s in self.systems.iter_mut() {
                s.run(&self.world);
            }
        }
    }

    #[derive(PartialEq, Debug)]
    struct CacheTest {
        data: u32,
    }
    impl Default for CacheTest {
        fn default() -> Self {
            Self { data: 6 }
        }
    }

    #[derive(Debug, Default)]
    struct Resource1 {
        _data: u32,
    }
    impl Resource for Resource1 {}

    #[derive(Debug)]
    struct Component1 {
        _data: u32,
    }
    impl Component for Component1 {}

    fn test() {
        println!("hello")
    }

    fn test1(_query: Query<(&Component1,)>) {}
    fn test2(_res: Res<Resource1>) {}
    fn test3(_res: Res<Resource1>, _query: Query<(&Component1,)>) {}
    fn test4(_local: Local<Resource1>, _query: Query<(&Component1,)>) {}

    #[test]
    fn test_executor() {
        Executor::default()
            .add_system(test)
            .add_system(test1)
            .add_system(test2)
            .add_system(test3)
            .add_system(test4)
            .run();
    }
}

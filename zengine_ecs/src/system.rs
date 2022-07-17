use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use zengine_macro::all_tuples;

use crate::{
    query::{Query, QueryCache, QueryParameters},
    world::{Resource, World},
};

pub trait SystemParam: Sized {
    type Fetch: for<'a> SystemParamFetch<'a> + Default;
}

pub trait SystemParamFetch<'a> {
    type Item;

    fn fetch(&'a mut self, world: &'a World) -> Self::Item;
}

pub type SystemParamItem<'a, P> = <<P as SystemParam>::Fetch as SystemParamFetch<'a>>::Item;

pub struct QueryState<T: QueryParameters> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: QueryParameters> Default for QueryState<T> {
    fn default() -> Self {
        QueryState {
            _marker: std::marker::PhantomData::default(),
        }
    }
}

impl<'a, T: QueryParameters> SystemParamFetch<'a> for QueryState<T> {
    type Item = Query<'a, T>;

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        Query {
            data: T::fetch(world, &mut None),
        }
    }
}

impl<'a, T: QueryParameters> SystemParam for Query<'a, T> {
    type Fetch = QueryState<T>;
}

type Res<'a, R> = RwLockReadGuard<'a, R>;

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
        world.get_resource().unwrap()
    }
}

impl<'a, R: Resource> SystemParam for Res<'a, R> {
    type Fetch = ResState<R>;
}

type ResMut<'a, R> = RwLockWriteGuard<'a, R>;

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
        world.get_mut_resource().unwrap()
    }
}

impl<'a, R: Resource> SystemParam for ResMut<'a, R> {
    type Fetch = ResMutState<R>;
}

type Local<'a, T> = &'a mut T;

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

pub trait SystemFunction<P: SystemParam> {
    fn run_function(&self, parameter: SystemParamItem<P>);
}

impl<Param: SystemParam, F> IntoSystem<Param> for F
where
    F: SystemFunction<Param>,
{
    type System = F;
    fn into_system(self) -> SystemWrapper<Self::System, Param> {
        SystemWrapper {
            _marker: std::marker::PhantomData::default(),
            function: self,
            param_state: Param::Fetch::default(),
        }
    }
}

trait IntoSystem<P: SystemParam> {
    type System: SystemFunction<P>;

    fn into_system(self) -> SystemWrapper<Self::System, P>;
}

struct SystemWrapper<F: SystemFunction<P>, P: SystemParam> {
    _marker: std::marker::PhantomData<P>,
    function: F,
    param_state: P::Fetch,
}

pub trait System {
    fn run(&mut self, world: &World);
}

impl<F: SystemFunction<P>, P: SystemParam> System for SystemWrapper<F, P> {
    fn run(&mut self, world: &World) {
        let data: <<P as SystemParam>::Fetch as SystemParamFetch>::Item =
            <P as SystemParam>::Fetch::fetch(&mut self.param_state, world);
        self.function.run_function(data);
    }
}

macro_rules! impl_system_function {
    () => {
        #[allow(non_snake_case)]
        impl<'a> SystemParamFetch<'a> for () {
            type Item = ();

            fn fetch(&mut self, _world: &'a World) -> Self::Item {
                ()
            }
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

            fn fetch(&'a mut self, world: &'a World) -> Self::Item {
                let ($($param,)*) = self;

                ($($param::fetch($param, world),)*)
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
    use std::{any::Any, marker::PhantomData};

    use crate::{
        component::Component,
        query::{Query, QueryCache},
        world::{self, Resource, World},
    };

    use super::{IntoSystem, Local, Res, System, SystemFunction, SystemParam, SystemWrapper};

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
        data: u32,
    }
    impl Resource for Resource1 {}

    #[derive(Debug)]
    struct Component1 {
        data: u32,
    }
    impl Component for Component1 {}

    fn test() {
        println!("hello")
    }

    fn test1(query: Query<(&Component1,)>) {}
    fn test2(res: Res<Resource1>) {}
    fn test3(res: Res<Resource1>, query: Query<(&Component1,)>) {}
    fn test4(local: Local<Resource1>, query: Query<(&Component1,)>) {}

    #[test]
    fn test_executor() {
        // let mut list: Vec<Box<dyn System>> = Vec::default();

        // let t = Box::new(SystemWrapper {
        //     system: test6,
        //     cache: (Resource1::default(), None),
        //     _marker: PhantomData::default(),
        // });

        // list.push(t);

        Executor::default()
            .add_system(test)
            .add_system(test1)
            .add_system(test2)
            .add_system(test3)
            .add_system(test4)
            .run();
    }
}

use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use zengine_macro::all_tuples;

use crate::{
    query::{Query, QueryCache, QueryParameters},
    world::{Resource, World},
};

pub trait SystemParam: Sized {
    type Fetch: for<'a> SystemParamFetch<'a> + Default;
}

//pub trait SystemParam: Sized + for<'a> SystemParamFetch<'a> {}

pub trait SystemParamFetch<'a> {
    type Item;
    //type Cache;
    //fn fetch(world: &'a World, cache: &'a mut Self::Cache) -> Self;
    fn fetch(&self, world: &'a World) -> Self::Item;
}

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
    //type Cache = Option<QueryCache>;
    // fn fetch(world: &'a World, cache: &'a mut Self::Cache) -> Self {
    //     if let Some(some_cache) = cache {
    //         if some_cache.last_archetypes_count != world.archetypes.len() {
    //             cache.take();
    //         }
    //     }
    //     Query {
    //         data: T::fetch(world, cache),
    //     }
    // }
    fn fetch(&self, world: &'a World) -> Self::Item {
        Query {
            data: T::fetch(world, &mut None),
        }
    }
}

impl<'a, T: QueryParameters> SystemParam for Query<'a, T> {
    type Fetch = QueryState<T>;
}

pub type SystemParamItem<'a, P> = <<P as SystemParam>::Fetch as SystemParamFetch<'a>>::Item;

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

type Res<'a, R> = RwLockReadGuard<'a, R>;
type ResMut<'a, R> = RwLockWriteGuard<'a, R>;

impl<'a, R: Resource> SystemParamFetch<'a> for ResState<R> {
    type Item = Res<'a, R>;
    //type Cache = ();
    // fn fetch(world: &'a World, _cache: &'a mut Self::Cache) -> Self {
    //     world.get_resource().unwrap()
    // }
    fn fetch(&self, world: &'a World) -> Self::Item {
        world.get_resource().unwrap()
    }
}
impl<'a, R: Resource> SystemParam for Res<'a, R> {
    type Fetch = ResState<R>;
}

type Local<'a, T> = &'a mut T;

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

impl<F: SystemFunction<P>, P: SystemParam> System for SystemWrapper<F, P> {
    fn run(&self, world: &World) {
        let data: <<P as SystemParam>::Fetch as SystemParamFetch>::Item =
            <P as SystemParam>::Fetch::fetch(&self.param_state, world);
        self.function.run_function(data);
    }
}

pub trait System {
    fn run(&self, world: &World);
}

macro_rules! impl_system_function {
    () => {
        #[allow(non_snake_case)]
        impl<'a> SystemParamFetch<'a> for () {
            type Item = ();
            fn fetch(&self, _world: &'a World) -> Self::Item {
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
            fn fetch(&self, world: &'a World) -> Self::Item {
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

    // trait Test {
    //     fn launch(&mut self, world: &World) {}
    // }

    // impl<S: AnySystemFunction<P, C>, P, C> Test for (S, C, PhantomData<P>) {
    //     fn launch(&mut self, world: &World) {
    //         //self.0.run_system(world, &mut self.1);
    //     }
    // }

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

    // struct TestArgs1 {}
    // impl<'a> SystemParam<'a> for TestArgs1 {
    //     type Cache = u32;
    //     fn fetch(_world: &World, cache: &mut Self::Cache) -> Self {
    //         println!("cache1 check");
    //         assert_eq!(*cache, 0);
    //         TestArgs1 {}
    //     }
    // }

    #[derive(PartialEq, Debug)]
    struct CacheTest {
        data: u32,
    }
    impl Default for CacheTest {
        fn default() -> Self {
            Self { data: 6 }
        }
    }
    // struct TestArgs2 {}
    // impl<'a> SystemParam<'a> for TestArgs2 {
    //     type Cache = CacheTest;
    //     fn fetch(_world: &World, cache: &mut Self::Cache) -> Self {
    //         println!("cache2 check");
    //         assert_eq!(*cache, CacheTest { data: 6 });
    //         TestArgs2 {}
    //     }
    // }

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

    // fn test2(_test1: TestArgs1) {
    //     println!("hello2")
    // }

    // fn test3(_test1: TestArgs1, _test2: TestArgs2) {
    //     println!("hello3")
    // }

    fn test4(query: Query<(&Component1,)>) {}
    fn test5(res: Res<Resource1>) {}
    fn test6(local: Res<Resource1>, query: Query<(&Component1,)>) {}

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
            .add_system(test4)
            .add_system(test5)
            .add_system(test6)
            .run();
    }
}

use std::{
    any::Any,
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use zengine_macro::all_tuples_with_idexes;

use crate::{
    query::{Query, QueryParameters},
    world::{Resource, World},
};

pub type SystemStorage = Vec<Box<dyn AnySystem>>;

pub trait SystemManager {
    fn add_system<'a, P: Any, C: Any + Default, S: System<P, C>>(self, system: S) -> Self;
}

pub trait AnySystem {
    fn run(&mut self, world: &World);
}

struct SystemFunction<S: System<P, C>, P, C: Default> {
    _marker: std::marker::PhantomData<P>,
    system: S,
    cache: C,
}

impl<S: System<P, C>, P, C: Default> SystemFunction<S, P, C> {
    pub fn run_function_system(&mut self, world: &World) {
        self.system.run_system(world, &mut self.cache);
    }
}

impl<S: System<P, C>, P, C: Default> AnySystem for SystemFunction<S, P, C> {
    fn run(&mut self, world: &World) {
        self.run_function_system(world)
    }
}

pub trait System<P, C>: Any {
    fn run_system(&self, world: &World, cache: &mut C);
}

pub trait AnySystemParam {
    type Cache;
    fn fetch(world: &World, cache: &mut Self::Cache) -> Self;
}

impl<P, C> AnySystemParam for P
where
    P: for<'a> SystemParam<'a, Cache = C>,
{
    type Cache = C;
    fn fetch(world: &World, cache: &mut Self::Cache) -> Self {
        P::fetch(world, cache)
    }
}

pub trait SystemParam<'a> {
    type Cache;
    fn fetch(world: &'a World, cache: &mut Self::Cache) -> Self;
}

impl<'a> SystemParam<'a> for () {
    type Cache = ();
    fn fetch(_world: &'a World, _cache: &mut Self::Cache) -> Self {}
}

impl<'a, T: QueryParameters> SystemParam<'a> for Query<'a, T> {
    type Cache = ();
    fn fetch(world: &'a World, cache: &mut Self::Cache) -> Self {
        world.query::<T>(None)
    }
}

type Res<'a, R> = RwLockReadGuard<'a, R>;
type ResMut<'a, R> = RwLockWriteGuard<'a, R>;

impl<'a, R: Resource> SystemParam<'a> for Res<'a, R> {
    type Cache = ();
    fn fetch(world: &'a World, _cache: &mut Self::Cache) -> Self {
        world.get_resource().unwrap()
    }
}

impl<'a, R: Resource> SystemParam<'a> for ResMut<'a, R> {
    type Cache = ();
    fn fetch(world: &'a World, _cache: &mut Self::Cache) -> Self {
        world.get_mut_resource().unwrap()
    }
}

macro_rules! impl_system_function {
    () => {
        impl< Sys: Fn() + 'static> System<(), ()> for Sys
        {
            fn run_system(&self, _world: &World, _cache: &mut ()) {
                (self)();
            }
        }
    };
    ($($param: ident => $index:tt),+) => {
        impl<$($param: AnySystemParam),*, Sys: Fn( $($param),* ) + 'static> System<((), $($param),*), ($($param::Cache),*,)> for Sys
        {
            fn run_system(&self, world: &World, cache: &mut ($($param::Cache),*,)) {
                (self)($($param::fetch(world, &mut cache.$index)),*);
            }
        }
    }
}
all_tuples_with_idexes!(impl_system_function, 0, 14, F);

#[cfg(test)]
mod tests {
    use std::{any::Any, marker::PhantomData};

    use crate::world::World;

    use super::{System, SystemFunction, SystemManager, SystemParam, SystemStorage};

    #[derive(Default)]
    struct Executor {
        world: World,
        systems: SystemStorage,
    }

    impl SystemManager for Executor {
        fn add_system<'a, P: Any, C: Any + Default, S: System<P, C>>(mut self, system: S) -> Self {
            self.systems.push(Box::new(SystemFunction {
                system,
                cache: C::default(),
                _marker: PhantomData::default(),
            }));

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

    struct TestArgs1 {}
    impl<'a> SystemParam<'a> for TestArgs1 {
        type Cache = u32;
        fn fetch(_world: &World, cache: &mut Self::Cache) -> Self {
            println!("cache1 check");
            assert_eq!(*cache, 0);
            TestArgs1 {}
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
    struct TestArgs2 {}
    impl<'a> SystemParam<'a> for TestArgs2 {
        type Cache = CacheTest;
        fn fetch(_world: &World, cache: &mut Self::Cache) -> Self {
            println!("cache2 check");
            assert_eq!(*cache, CacheTest { data: 6 });
            TestArgs2 {}
        }
    }

    fn test() {
        println!("hello")
    }

    fn test2(_test1: TestArgs1) {
        println!("hello2")
    }

    fn test3(_test1: TestArgs1, _test2: TestArgs2) {
        println!("hello3")
    }

    #[test]
    fn test_executor() {
        Executor::default()
            .add_system(test)
            .add_system(test2)
            .add_system(test3)
            .run();
    }
}

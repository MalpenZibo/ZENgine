use std::{
    any::Any,
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use zengine_macro::all_tuples;

use crate::{
    query::{Query, QueryParameters},
    world::{Resource, World},
};

pub type SystemStorage = Vec<Box<dyn AnySystem>>;

pub trait SystemManager {
    fn add_system<'a, P: Any, S: System<P>>(self, system: S) -> Self;
}

pub trait AnySystem {
    fn run(&self, world: &World);
}

struct SystemFunction<S: System<P>, P> {
    _marker: std::marker::PhantomData<P>,
    system: S,
}

impl<S: System<P>, P> SystemFunction<S, P> {
    pub fn run(&self, world: &World) {
        self.system.run_system(world)
    }
}

impl<S: System<P>, P> AnySystem for SystemFunction<S, P> {
    fn run(&self, world: &World) {
        self.run(world)
    }
}

pub trait System<P>: Any {
    fn run_system(&self, world: &World);
}

pub trait AnySystemParam {
    fn fetch(world: &World) -> Self;
}

impl<P> AnySystemParam for P
where
    P: for<'a> SystemParam<'a>,
{
    fn fetch(world: &World) -> Self {
        P::fetch(world)
    }
}

pub trait SystemParam<'a> {
    fn fetch(world: &'a World) -> Self;
}

impl<'a> SystemParam<'a> for () {
    fn fetch(_world: &'a World) -> Self {}
}

impl<'a, T: QueryParameters> SystemParam<'a> for Query<'a, T> {
    fn fetch(world: &'a World) -> Self {
        world.query::<T>()
    }
}

type Res<'a, R> = RwLockReadGuard<'a, R>;
type ResMut<'a, R> = RwLockWriteGuard<'a, R>;

impl<'a, R: Resource> SystemParam<'a> for Res<'a, R> {
    fn fetch(world: &'a World) -> Self {
        world.get_resource().unwrap()
    }
}

impl<'a, R: Resource> SystemParam<'a> for ResMut<'a, R> {
    fn fetch(world: &'a World) -> Self {
        world.get_mut_resource().unwrap()
    }
}

macro_rules! impl_system_function {
    () => {
        impl< Sys: Fn() + 'static> System<()> for Sys
        {
            fn run_system(&self, _world: &World) {
                (self)();
            }
        }
    };
    ($($param: ident),+) => {
        impl<$($param: AnySystemParam),*, Sys: Fn( $($param),* ) + 'static> System<((), $($param),*)> for Sys
        {
            fn run_system(&self, world: &World) {
                (self)($($param::fetch(world)),*);
            }
        }
    }
}
all_tuples!(impl_system_function, 0, 14, F);

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
        fn add_system<'a, P: Any, S: System<P>>(mut self, system: S) -> Self {
            self.systems.push(Box::new(SystemFunction {
                system,
                _marker: PhantomData::default(),
            }));

            self
        }
    }

    impl Executor {
        pub fn run(self) {
            for s in self.systems.iter() {
                s.run(&self.world);
            }
        }
    }

    struct TestArgs1 {}
    impl<'a> SystemParam<'a> for TestArgs1 {
        fn fetch(_world: &World) -> Self {
            TestArgs1 {}
        }
    }

    struct TestArgs2 {}
    impl<'a> SystemParam<'a> for TestArgs2 {
        fn fetch(_world: &World) -> Self {
            TestArgs2 {}
        }
    }

    fn test() {
        println!("hello")
    }

    fn test2(_test1: TestArgs1) {
        println!("hello2")
    }

    fn test3(_test1: TestArgs1, _test2: TestArgs1) {
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

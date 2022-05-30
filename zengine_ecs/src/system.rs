use std::any::Any;

use zengine_macro::all_tuples;

use crate::world::World;

pub type SystemStorage = Vec<Box<dyn AnySystem>>;

pub trait SystemManager {
    fn add_system<P: Any, S: System<P>>(self, system: S) -> Self;
}

pub trait AnySystem {
    fn run(&self, world: &World);
}

pub trait System<P>: Any {
    fn run(&self, world: &World);
}

struct SystemFunction<S: System<P>, P> {
    _marker: std::marker::PhantomData<P>,
    system: S,
}

impl<S: System<P>, P> SystemFunction<S, P> {
    pub fn run(&self, world: &World) {
        self.system.run(world)
    }
}

impl<S: System<P>, P> AnySystem for SystemFunction<S, P> {
    fn run(&self, world: &World) {
        self.run(world)
    }
}

pub trait SystemParam {
    fn fetch(world: &World) -> Self;
}

impl SystemParam for () {
    fn fetch(_world: &World) -> Self {}
}

macro_rules! impl_system_function {
    () => {
        impl<Sys: Fn() + 'static> System<()> for Sys
        {
            fn run(&self, _world: &World) {
                (self)();
            }
        }
    };
    ($($param: ident),+) => {
        impl<$($param: SystemParam),*, Sys: Fn( $($param),* ) + 'static> System<((), $($param),*)> for Sys
        {
            fn run(&self, world: &World) {
                (self)($($param::fetch(world)),*);
            }
        }
    }
}

all_tuples!(impl_system_function, 0, 26, F);

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
        fn add_system<P: Any, S: System<P>>(mut self, system: S) -> Self {
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
    impl SystemParam for TestArgs1 {
        fn fetch(_world: &World) -> Self {
            TestArgs1 {}
        }
    }

    struct TestArgs2 {}
    impl SystemParam for TestArgs2 {
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

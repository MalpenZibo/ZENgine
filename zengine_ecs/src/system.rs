use std::any::Any;

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

trait SystemParam {
    fn fetch(world: &World) -> Self;
}

impl SystemParam for () {
    fn fetch(_world: &World) -> Self {}
}

macro_rules! impl_system_function {
    () => {
        impl<Sys: Fn() -> () + 'static> System<()> for Sys
        {
            fn run(&self, _world: &World) {
                (self)();
            }
        }
    };
    ($($param: ident),+) => {
        impl<$($param: SystemParam),*, Sys: Fn( $($param),* ) -> () + 'static> System<((), $($param),*)> for Sys
        {
            fn run(&self, world: &World) {
                (self)($($param::fetch(world)),*);
            }
        }
    }
}
impl_system_function!();
impl_system_function!(A);
impl_system_function!(A, B);
impl_system_function!(A, B, C);
impl_system_function!(A, B, C, D);
impl_system_function!(A, B, C, D, E);
impl_system_function!(A, B, C, D, E, F);
impl_system_function!(A, B, C, D, E, F, G);
impl_system_function!(A, B, C, D, E, F, G, H);
impl_system_function!(A, B, C, D, E, F, G, H, I);
impl_system_function!(A, B, C, D, E, F, G, H, I, J);
impl_system_function!(A, B, C, D, E, F, G, H, I, J, K);
impl_system_function!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_system_function!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_system_function!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_system_function!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_system_function!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_system_function!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_system_function!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_system_function!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_system_function!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_system_function!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
impl_system_function!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
impl_system_function!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
impl_system_function!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
impl_system_function!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
impl_system_function!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);

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
        fn fetch(world: &World) -> Self {
            TestArgs1 {}
        }
    }

    struct TestArgs2 {}
    impl SystemParam for TestArgs2 {
        fn fetch(world: &World) -> Self {
            TestArgs2 {}
        }
    }

    fn test() {
        println!("hello")
    }

    fn test2(test1: TestArgs1) {
        println!("hello2")
    }

    fn test3(test1: TestArgs1, test2: TestArgs1) {
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

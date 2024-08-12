use std::sync::RwLockReadGuard;

use crate::{Resource, World};
use zengine_macro::all_tuples;

pub mod system_parameter;

pub trait System: Send + Sync {
    fn init(&mut self, world: &mut World);

    fn run(&mut self, world: &World);

    fn apply(&mut self, world: &mut World);
}

pub trait SystemFunction<Marker>: Send + Sync + 'static {
    fn run_function(&mut self, world: &World);

    fn into_system(self) -> BoxedSystem;
}

pub struct SystemFunctionData<Marker, F: SystemFunction<Marker>> {
    function: F,
    _phantom: std::marker::PhantomData<fn() -> Marker>,
}

pub type BoxedSystem = Box<dyn System>;

impl<Marker, F: SystemFunction<Marker>> System for SystemFunctionData<Marker, F> {
    fn init(&mut self, world: &mut World) {
        // self.param_state.init(world);
    }

    fn run(&mut self, world: &World) {
        // let data: <<P as SystemParam>::Fetch as SystemParamFetch>::Item = self.param_state.fetch(world);
        self.function.run_function(world);
    }

    fn apply(&mut self, world: &mut World) {
        // self.param_state.apply(world);
    }
}

pub trait SystemParam: Sync + Sized {
    type State: Send + Sync;
    type Item<'a>: Sync;

    fn get(world: &World) -> Self::Item<'_>;
}

pub struct Res<'a, R: Resource>(RwLockReadGuard<'a, R>);
unsafe impl<'a, T: Resource> Send for Res<'a, T> {}

impl<'a, T: Resource> SystemParam for Res<'a, T> {
    type State = T;
    type Item<'b> = Res<'b, T>;

    fn get(world: &World) -> Res<'_, T> {
        Res(world.get_resource::<T>().unwrap())
    }
}

impl<A: SystemParam, B: SystemParam> SystemParam for (A, B) {
    type State = (A::State, B::State);
    type Item<'a> = (A::Item<'a>, B::Item<'a>);

    fn get(world: &World) -> Self::Item<'_> {
        (A::get(world), B::get(world))
    }
}

macro_rules! impl_system_function {
    ($($param: ident),*) => {
        #[allow(non_snake_case)]
        impl<F: Send + Sync + 'static, $($param: SystemParam + 'static),*> SystemFunction<fn($($param,)*)> for F
        where
            F: FnMut($($param),*) + FnMut($($param::Item<'_>),*)
        {
            fn run_function(&mut self, _world: &World) {
                (self)($($param::get(_world)),*);
            }

            fn into_system(self) -> BoxedSystem {
                Box::new(SystemFunctionData {
                    function: self,
                    _phantom: std::marker::PhantomData,
                })
            }
        }
    }
}
all_tuples!(impl_system_function, 0, 12, F);

#[cfg(test)]
mod tests {
    use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

    use crate::Resource;

    use super::{BoxedSystem, Res, SystemFunction};

    #[derive(Debug)]
    struct T {
        a: i32,
    }
    impl Resource for T {}

    #[derive(Debug)]
    struct W {
        a: i32,
    }
    impl Resource for W {}

    fn system0() {}

    fn system1(a: Res<T>) {
        println!("{:?}", a.0);
    }

    fn system2(a: Res<T>, b: Res<W>) {
        println!("{:?}", a.0);
    }

    #[test]
    fn test_system() {
        let sys0: BoxedSystem = system0.into_system();
        let sys1: BoxedSystem = system1.into_system();
        let sys2: BoxedSystem = system2.into_system();
        let mut systems: Vec<BoxedSystem> = vec![sys0, sys1, sys2];

        let world = Default::default();

        systems.par_iter_mut().for_each(|s| {
            s.run(&world);
        });
    }
}

use crate::World;
use system_parameter::SystemParam;
use zengine_macro::all_tuples;

pub mod system_parameter;

pub trait System: Send + Sync {
    fn init(&mut self, world: &mut World);

    fn run(&mut self, world: &World);

    fn apply(&mut self, world: &mut World);
}

pub trait SystemFunction<Marker>: Send + Sync + 'static {
    type Param: SystemParam;

    fn run_function(&mut self, data: <Self::Param as SystemParam>::Item<'_, '_>);

    fn into_system(self) -> BoxedSystem;
}

pub struct SystemFunctionData<Marker, F: SystemFunction<Marker>> {
    function: F,
    param_state: <F::Param as SystemParam>::State,
    _phantom: std::marker::PhantomData<fn() -> Marker>,
}

pub type BoxedSystem = Box<dyn System>;

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
            F: FnMut($($param),*) + FnMut($($param::Item<'_, '_>),*)
        {
            type Param = ($($param,)*);

            fn run_function(&mut self, data: <Self::Param as SystemParam>::Item<'_, '_>) {
                let ($($param,)*) = data;
                (self)($($param),*);
            }

            fn into_system(self) -> BoxedSystem {
                Box::new(SystemFunctionData {
                    function: self,
                    param_state: Default::default(),
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

    use super::{BoxedSystem, system_parameter::Res, SystemFunction};

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

    fn system2(a: Res<T>, b: Res<W>) {
        println!("{:?}", a.a);
        println!("{:?}", b.a);
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

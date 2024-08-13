use crate::World;

use super::{ConditionParam, ConditionParamFetch, ConditionParamItem};

/// A trait implemented for all functions that can be used as a [Condition]
pub trait ConditionFunction<P: ConditionParam> {
    fn run_function(&self, parameter: ConditionParamItem<P>) -> bool;
}

/// Wraps a function that implements the [SystemFunction] trait
pub struct ConditionWrapper<F: ConditionFunction<P>, P: ConditionParam> {
    _marker: std::marker::PhantomData<P>,
    function: F,
    param_state: P::Fetch,
}

// Condition trait
pub trait Condition {
    fn init(&mut self, world: &mut World);

    fn run(&mut self, world: &World) -> bool;
}

impl<F: ConditionFunction<P>, P: ConditionParam> Condition for ConditionWrapper<F, P> {
    fn init(&mut self, world: &mut World) {
        self.param_state.init(world);
    }

    fn run(&mut self, world: &World) -> bool {
        let data: <<P as ConditionParam>::Fetch as ConditionParamFetch>::Item =
            <P as ConditionParam>::Fetch::fetch(&mut self.param_state, world);

        self.function.run_function(data)
    }
}

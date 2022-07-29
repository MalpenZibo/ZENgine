use crate::world::World;

mod command;
mod event_parameter;
mod local_parameter;
mod query_parameter;
mod res_parameter;

pub use command::*;
pub use event_parameter::*;
pub use local_parameter::*;
pub use query_parameter::*;
pub use res_parameter::*;

pub trait SystemParam: Sized {
    type Fetch: for<'a> SystemParamFetch<'a> + Default;
}

pub trait SystemParamFetch<'a> {
    type Item;

    fn init(&mut self, _world: &mut World) {}

    fn fetch(&'a mut self, world: &'a World) -> Self::Item;

    fn apply(&mut self, _world: &mut World) {}
}

pub type SystemParamItem<'a, P> = <<P as SystemParam>::Fetch as SystemParamFetch<'a>>::Item;

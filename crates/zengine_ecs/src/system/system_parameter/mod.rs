use crate::World;

mod command;
mod event_parameter;
mod query_parameter;
mod res_parameter;

pub use command::*;
pub use event_parameter::*;
pub use res_parameter::*;

pub trait SystemParam: Sync + Sized {
    type State: Default + Send + Sync;
    type Item<'w, 's>: Sync;

    fn init(_world: &mut World, _state: &mut Self::State) {}

    fn get<'w, 's>(world: &'w World, state: &'s mut Self::State) -> Self::Item<'w, 's>;

    fn apply(_world: &mut World, _state: &mut Self::State) {}
}

pub trait ReadOnlySystemParam: SystemParam {}

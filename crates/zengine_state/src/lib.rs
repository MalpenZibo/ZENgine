use std::fmt::Debug;

use zengine_ecs::system::ResMut;
use zengine_engine::{Engine, Stage};
use zengine_macro::Resource;

pub trait States: Copy + Default + Debug + Send + Sync + 'static {}

#[derive(Resource, Default, Debug)]
pub struct State<S: States> {
    current: S,
    pending: Option<S>,
}

impl<S: States> State<S> {
    pub fn get(&self) -> S {
        self.current
    }

    pub fn pending(&self) -> Option<S> {
        self.pending
    }

    pub fn next(&mut self, next: S) {
        self.pending = Some(next);
    }

    pub fn reset(&mut self) {
        self.pending = None;
    }
}

pub trait StateExtension {
    fn init_state<S: States>(&mut self) -> &mut Self;

    fn insert_state<S: States>(&mut self, state: S) -> &mut Self;
}

impl StateExtension for Engine {
    fn init_state<S: States>(&mut self) -> &mut Self {
        self.world.create_resource(State {
            current: S::default(),
            pending: None,
        });
        self.add_system_into_stage(state_transition::<S>, Stage::PreUpdate);

        self
    }

    fn insert_state<S: States>(&mut self, state: S) -> &mut Self {
        self.world.create_resource(State {
            current: state,
            pending: None,
        });
        self.add_system_into_stage(state_transition::<S>, Stage::PreUpdate);

        self
    }
}

fn state_transition<S: States>(mut state: ResMut<State<S>>) {
    if let Some(next) = state.pending.take() {
        state.current = next
    }
}

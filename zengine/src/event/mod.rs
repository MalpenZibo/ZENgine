mod stream;

pub use self::stream::EventStream;
pub use self::stream::SubscriptionToken;

use crate::core::Resource;
use crate::event::input::Input;
use fnv::FnvHashMap;
use serde::Deserialize;
use std::any::Any;
use std::hash::Hash;

pub mod input;
mod input_system;

pub use self::input_system::InputSystem;

pub trait InputType: Any + Eq + PartialEq + Hash + Clone {}

impl InputType for String {}

#[derive(Debug, Deserialize, PartialEq)]
pub struct AxisBind {
    source: Input,
    scale: f32,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ActionBind {
    source: Input,
}

#[derive(Default, Deserialize, PartialEq)]
pub struct Bindings<T: InputType> {
    pub axis_mappings: Option<FnvHashMap<T, Vec<AxisBind>>>,
    pub action_mappings: Option<FnvHashMap<T, Vec<ActionBind>>>,
}

pub struct InputHandler<T: InputType> {
    actions_value: FnvHashMap<T, Vec<(Input, bool)>>,
    axes_value: FnvHashMap<T, Vec<(Input, f32)>>,
}
impl<T: InputType> Default for InputHandler<T> {
    fn default() -> Self {
        InputHandler {
            actions_value: FnvHashMap::default(),
            axes_value: FnvHashMap::default(),
        }
    }
}

impl<T: InputType> Resource for InputHandler<T> {}

impl<T: InputType> InputHandler<T> {
    pub fn axis_value(&self, input_type: T) -> f32 {
        self.axes_value
            .get(&input_type)
            .map(|entry| entry.iter().last().map(|last_event| last_event.1))
            .flatten()
            .unwrap_or(0.0)
    }

    pub fn action_value(&self, input_type: T) -> bool {
        self.actions_value
            .get(&input_type)
            .map(|entry| entry.iter().last().map(|last_event| last_event.1))
            .flatten()
            .unwrap_or(false)
    }
}

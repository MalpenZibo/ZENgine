use crate::core::system::Read;
use crate::core::system::System;
use crate::core::system::Write;
use crate::core::Resource;
use crate::core::Store;
use crate::event::event::Input;
use crate::event::event::InputEvent;
use crate::event::event_stream::{EventStream, SubscriptionToken};
use fnv::FnvHashMap;
use serde::Deserialize;
use std::any::Any;
use std::hash::Hash;

pub trait InputType: Any + Eq + PartialEq + Hash + Clone {}

impl InputType for String {}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Axis {
    source: Input,
    scale: f32,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Action {
    source: Input,
}

#[derive(Default, Deserialize, PartialEq)]
pub struct Bindings<T: InputType> {
    pub axis_mappings: FnvHashMap<T, Vec<Axis>>,
    pub action_mappings: FnvHashMap<T, Vec<Action>>,
}

#[derive(Default)]
pub struct InputSystem<T: InputType> {
    input_stream_token: Option<SubscriptionToken>,
    bindings: Bindings<T>,
}

pub struct InputHandler<T: InputType> {
    actions_value: FnvHashMap<T, bool>,
    axes_value: FnvHashMap<T, f32>,
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
    pub fn axis_value(&self, input_type: T) -> Option<f32> {
        self.axes_value.get(&input_type).map(|value| value.clone())
    }

    pub fn action_value(&self, input_type: T) -> Option<bool> {
        self.actions_value
            .get(&input_type)
            .map(|value| value.clone())
    }
}

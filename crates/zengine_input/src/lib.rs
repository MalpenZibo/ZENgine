use fnv::FnvHashMap;
use serde::Deserialize;
use std::any::Any;
use std::hash::Hash;
use zengine_ecs::Resource;

mod input;
mod input_system;

pub use input::*;
pub use input_system::*;

pub mod device {
    use serde::Deserialize;

    pub type Key = winit::event::VirtualKeyCode;

    pub type MouseButton = winit::event::MouseButton;

    pub type ControllerButton = gilrs::Button;

    #[derive(Eq, PartialEq, Hash, Debug, Deserialize, Clone)]
    pub enum Which {
        Left,
        Right,
    }
}

pub trait InputType: Any + Eq + PartialEq + Hash + Clone + Default + std::fmt::Debug {}

impl InputType for String {}

#[derive(Debug, Deserialize, PartialEq)]
pub struct AxisBind {
    source: Input,
    #[serde(default)]
    invert: bool,
    #[serde(default)]
    discrete_map: Option<f32>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ActionBind {
    source: Input,
}

#[derive(Default, Deserialize, PartialEq)]
pub struct Bindings<T: InputType> {
    pub axis_mappings: Option<FnvHashMap<T, Vec<AxisBind>>>,
    pub action_mappings: Option<FnvHashMap<T, Vec<ActionBind>>>,
}

#[derive(Debug)]
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

impl<T: InputType + Send + Sync + std::fmt::Debug> Resource for InputHandler<T> {}

impl<T: InputType> InputHandler<T> {
    pub fn axis_value(&self, input_type: T) -> f32 {
        self.axes_value
            .get(&input_type)
            .and_then(|entry| entry.iter().last().map(|last_event| last_event.1))
            .unwrap_or(0.0)
    }

    pub fn action_value(&self, input_type: T) -> bool {
        self.actions_value
            .get(&input_type)
            .and_then(|entry| entry.iter().last().map(|last_event| last_event.1))
            .unwrap_or(false)
    }
}

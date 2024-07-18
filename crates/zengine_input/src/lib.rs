use crate::input_system::input_system;
use fnv::FnvHashMap;
use serde::Deserialize;
use std::any::Any;
use std::hash::Hash;
use zengine_engine::{Module, Stage};
use zengine_macro::Resource;

mod input;
mod input_system;

pub use input::*;

/// Adds input bindings mapping support to the engine
pub struct InputModule<T: InputType>(pub Bindings<T>);

impl<T: InputType> Module for InputModule<T> {
    fn init(self, engine: &mut zengine_engine::Engine) {
        engine.add_system_into_stage(input_system(self.0), Stage::PreUpdate);
    }
}

/// Collection of enums that rappresent information about input devices
pub mod device {
    use serde::Deserialize;

    /// enum with all possible keyboard keys
    pub type Key = winit::keyboard::KeyCode;

    /// enum with all possible mouse buttons
    pub type MouseButton = winit::event::MouseButton;

    /// enum with all possible gamepad buttons
    pub type ControllerButton = gilrs::Button;

    /// enum that rappresent wich gamepad stick or trigger has been used
    #[derive(Eq, PartialEq, Hash, Debug, Deserialize, Clone)]
    pub enum Which {
        Left,
        Right,
    }

    /// enum that rappresent touch-screen input state.
    pub type TouchPhase = winit::event::TouchPhase;
}

/// Rappresent a possible input key binding.
///
/// This trait is implemented for [String] type
/// but it's strongly suggested to implement this InputType
/// using the provided `InputType` derive macro
///
/// # Example
/// ```
/// use serde::Deserialize;
/// use zengine_macro::InputType;
///
/// #[derive(Deserialize, InputType, Hash, Eq, PartialEq, Clone, Default, Debug)]
/// pub enum UserInput {
///     #[default]
///     MoveLeft,
///     MoveRight,
///     Jump,
///     Fire
/// }
/// ```
pub trait InputType:
    Any + Eq + PartialEq + Hash + Clone + Send + Sync + Default + std::fmt::Debug
{
}

impl InputType for String {}

/// Bind an [InputType] to an [Input] source using an AxisBind
///
/// An axis bind gives an input value between -1.0 and 1.0
#[derive(Debug, Deserialize, PartialEq)]
pub struct AxisBind {
    source: Input,
    #[serde(default)]
    invert: bool,
    #[serde(default)]
    discrete_map: Option<f32>,
}

impl AxisBind {
    /// Creates an [AxisBind] with the given [Input]
    pub fn with_source(source: Input) -> Self {
        Self {
            source,
            invert: false,
            discrete_map: None,
        }
    }

    /// Invert the input value
    pub fn invert_input(mut self) -> Self {
        self.invert = true;

        self
    }

    /// Adds the given discrete map
    pub fn with_discrete_map(mut self, discrete_map: f32) -> Self {
        self.discrete_map = Some(discrete_map);

        self
    }
}

/// Bind an [InputType] to an [Input] source using an ActionBind
///
/// An action bind gives an input value that could be true or false
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ActionBind {
    source: Input,
}

impl ActionBind {
    /// Creates an [ActionBind] with the given [Input]
    pub fn with_source(source: Input) -> Self {
        Self { source }
    }
}

/// Contains all the defined input bindings for the [InputModule]
#[derive(Default, Deserialize, PartialEq)]
pub struct Bindings<T: InputType> {
    pub axis_mappings: Option<FnvHashMap<T, Vec<AxisBind>>>,
    pub action_mappings: Option<FnvHashMap<T, Vec<ActionBind>>>,
}

impl<T: InputType> Bindings<T> {
    /// Adds the given [ActionBind] to the input [Bindings] for the given [InputType]
    pub fn add_action(mut self, input_type: T, bind: ActionBind) -> Self {
        match &mut self.action_mappings {
            Some(action_mappings) => match action_mappings.get_mut(&input_type) {
                Some(mappings) => {
                    mappings.push(bind);
                }
                None => {
                    action_mappings.insert(input_type, vec![bind]);
                }
            },
            None => {
                let mut action_mappings = FnvHashMap::default();
                action_mappings.insert(input_type, vec![bind]);

                self.action_mappings = Some(action_mappings);
            }
        };

        self
    }

    /// Adds the given [AxisBind] to the input [Bindings] for the given [InputType]
    pub fn add_axis(mut self, input_type: T, bind: AxisBind) -> Self {
        match &mut self.axis_mappings {
            Some(axis_mappings) => match axis_mappings.get_mut(&input_type) {
                Some(mappings) => {
                    mappings.push(bind);
                }
                None => {
                    axis_mappings.insert(input_type, vec![bind]);
                }
            },
            None => {
                let mut axis_mappings = FnvHashMap::default();
                axis_mappings.insert(input_type, vec![bind]);

                self.axis_mappings = Some(axis_mappings);
            }
        };

        self
    }
}

/// A [Resource](zengine_ecs::Resource) that handle the mapping between
/// the user input events and the [Bindings]
#[derive(Resource, Debug)]
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

impl<T: InputType> InputHandler<T> {
    /// Gets the current axis value for a given [InputType]
    pub fn axis_value(&self, input_type: T) -> f32 {
        self.axes_value
            .get(&input_type)
            .and_then(|entry| entry.iter().last().map(|last_event| last_event.1))
            .unwrap_or(0.0)
    }

    /// Gets the current action value for a given [InputType]
    pub fn action_value(&self, input_type: T) -> bool {
        self.actions_value
            .get(&input_type)
            .and_then(|entry| entry.iter().last().map(|last_event| last_event.1))
            .unwrap_or(false)
    }
}

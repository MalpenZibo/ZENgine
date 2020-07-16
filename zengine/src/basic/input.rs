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
    actions_value: FnvHashMap<T, f32>,
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

impl<T: InputType> InputSystem<T> {
    pub fn from_bindings(bindings: Bindings<T>) -> Self {
        InputSystem {
            input_stream_token: None,
            bindings: bindings,
        }
    }
}

impl<'a, T: InputType> System<'a> for InputSystem<T> {
    type Data = (
        Read<'a, EventStream<InputEvent>>,
        Write<'a, InputHandler<T>>,
    );

    fn init(&mut self, store: &mut Store) {
        if let Some(mut input_stream) = store.get_resource_mut::<EventStream<InputEvent>>() {
            self.input_stream_token = Some(input_stream.subscribe());
        }

        store.insert_resource(InputHandler::<T>::default());
    }

    fn run(&mut self, (event_stream, mut input_handler): Self::Data) {
        if let Some(token) = self.input_stream_token {
            for e in event_stream.read(&token) {
                for actions in self.bindings.action_mappings.iter() {
                    if let Some(_action) = actions.1.iter().find(|action| action.source == e.input)
                    {
                        input_handler
                            .actions_value
                            .insert(actions.0.clone(), e.value);
                    }
                }
                for axes in self.bindings.axis_mappings.iter() {
                    if let Some(axis) = axes.1.iter().find(|axis| axis.source == e.input) {
                        input_handler
                            .axes_value
                            .insert(axes.0.clone(), e.value * axis.scale);
                    }
                }
            }
        }
    }

    fn dispose(&mut self, store: &mut Store) {
        if let Some(token) = self.input_stream_token {
            if let Some(mut input_stream) = store.get_resource_mut::<EventStream<InputEvent>>() {
                input_stream.unsubscribe(token);
                self.input_stream_token = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::keyboard::Key;

    #[derive(Eq, PartialEq, Hash, Deserialize, Clone)]
    enum UserInput {
        Jump,
        X,
    }

    impl InputType for UserInput {}

    #[test]
    fn decode_typed_bindings_from_yaml() {
        let content = "
            axis_mappings: 
                X:
                - source:
                    Keyboard:
                        key: D
                  scale: 1.0
                - source:
                    Keyboard:
                        key: A
                  scale: -1.0
            action_mappings:
                Jump:
                - source: 
                    Keyboard: 
                        key: Space
        ";

        let bindings: Bindings<UserInput> = serde_yaml::from_str(&content).unwrap();

        assert_eq!(
            bindings.action_mappings.get(&UserInput::Jump).unwrap(),
            &vec!(Action {
                source: Input::Keyboard { key: Key::Space }
            })
        );

        assert_eq!(
            bindings.axis_mappings.get(&UserInput::X).unwrap(),
            &vec!(
                Axis {
                    source: Input::Keyboard { key: Key::D },
                    scale: 1.0
                },
                Axis {
                    source: Input::Keyboard { key: Key::A },
                    scale: -1.0
                }
            )
        );

        InputSystem::from_bindings(bindings);
    }

    #[test]
    fn decode_generics_bindings_from_yaml() {
        let content = "
            axis_mappings: 
                X:
                - source:
                    Keyboard:
                        key: D
                  scale: 1.0
                - source:
                    Keyboard:
                        key: A
                  scale: -1.0
            action_mappings:
                Jump:
                - source: 
                    Keyboard: 
                        key: Space
        ";

        let bindings: Bindings<String> = serde_yaml::from_str(&content).unwrap();

        assert_eq!(
            bindings.action_mappings.get("Jump").unwrap(),
            &vec!(Action {
                source: Input::Keyboard { key: Key::Space }
            })
        );

        assert_eq!(
            bindings.axis_mappings.get("X").unwrap(),
            &vec!(
                Axis {
                    source: Input::Keyboard { key: Key::D },
                    scale: 1.0
                },
                Axis {
                    source: Input::Keyboard { key: Key::A },
                    scale: -1.0
                }
            )
        );

        InputSystem::from_bindings(bindings);
    }
}

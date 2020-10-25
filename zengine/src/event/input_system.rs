use crate::core::system::{Read, System, Write};
use crate::core::Store;
use crate::event::input::InputEvent;
use crate::event::EventStream;
use crate::event::SubscriptionToken;
use crate::event::{Bindings, InputHandler, InputType};

#[derive(Default)]
pub struct InputSystem<T: InputType> {
    input_stream_token: Option<SubscriptionToken>,
    bindings: Bindings<T>,
    events: Vec<InputEvent>,
}

impl<T: InputType> InputSystem<T> {
    pub fn new(bindings: Bindings<T>) -> Self {
        InputSystem {
            input_stream_token: None,
            bindings,
            events: Vec::default(),
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
                if e.value > 0.0 {
                    match self
                        .events
                        .iter_mut()
                        .find(|ref event| event.input == e.input)
                    {
                        Some(event) => {
                            event.value = e.value;
                        }
                        None => {
                            self.events.push(InputEvent {
                                input: e.input.clone(),
                                value: e.value.clone(),
                            });
                        }
                    }
                } else {
                    self.events.retain(|events| events.input != e.input);
                }

                for actions in self.bindings.action_mappings.iter() {
                    if let Some(event) = self
                        .events
                        .iter()
                        .filter(|event| actions.1.iter().any(|action| action.source == event.input))
                        .last()
                    {
                        input_handler
                            .actions_value
                            .insert(actions.0.clone(), event.value > 0.0);
                    } else {
                        input_handler.actions_value.insert(actions.0.clone(), false);
                    }
                }
                for axes in self.bindings.axis_mappings.iter() {
                    if let Some(event) = self
                        .events
                        .iter()
                        .filter(|event| axes.1.iter().any(|action| action.source == event.input))
                        .last()
                    {
                        input_handler.axes_value.insert(
                            axes.0.clone(),
                            event.value
                                * (axes
                                    .1
                                    .iter()
                                    .find(|axis| axis.source == event.input)
                                    .map(|axis| axis.scale)
                                    .unwrap_or_else(|| 0.0)),
                        );
                    } else {
                        input_handler.axes_value.insert(axes.0.clone(), 0.0);
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
    use crate::core::system::AnySystem;
    use crate::device::keyboard::Key;
    use crate::event::input::Input;
    use crate::event::{ActionBind, AxisBind};
    use serde::Deserialize;

    #[derive(Eq, PartialEq, Hash, Deserialize, Clone)]
    enum UserInput {
        Jump,
        X,
    }

    impl InputType for UserInput {}

    fn setup_test() -> (Store, InputSystem<UserInput>) {
        let mut store = Store::default();
        store.insert_resource(EventStream::<InputEvent>::default());

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

        let input_system = InputSystem::<UserInput>::new(bindings);

        (store, input_system)
    }

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
            &vec!(ActionBind {
                source: Input::Keyboard { key: Key::Space }
            })
        );

        assert_eq!(
            bindings.axis_mappings.get(&UserInput::X).unwrap(),
            &vec!(
                AxisBind {
                    source: Input::Keyboard { key: Key::D },
                    scale: 1.0
                },
                AxisBind {
                    source: Input::Keyboard { key: Key::A },
                    scale: -1.0
                }
            )
        );

        InputSystem::new(bindings);
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
            &vec!(ActionBind {
                source: Input::Keyboard { key: Key::Space }
            })
        );

        assert_eq!(
            bindings.axis_mappings.get("X").unwrap(),
            &vec!(
                AxisBind {
                    source: Input::Keyboard { key: Key::D },
                    scale: 1.0
                },
                AxisBind {
                    source: Input::Keyboard { key: Key::A },
                    scale: -1.0
                }
            )
        );

        InputSystem::new(bindings);
    }

    #[test]
    fn retrieve_action_value() {
        let (mut store, mut input_system) = setup_test();

        AnySystem::init(&mut input_system, &mut store);
        {
            {
                let mut input_stream = store.get_resource_mut::<EventStream<InputEvent>>().unwrap();
                input_stream.publish(InputEvent {
                    input: Input::Keyboard { key: Key::Space },
                    value: 1.0,
                });
            }
            AnySystem::run(&mut input_system, &store);
            let input_handler = store.get_resource::<InputHandler<UserInput>>().unwrap();
            assert_eq!(input_handler.action_value(UserInput::Jump), Some(true));
        }

        AnySystem::dispose(&mut input_system, &mut store);
    }

    #[test]
    fn retrieve_axis_value() {
        let (mut store, mut input_system) = setup_test();

        AnySystem::init(&mut input_system, &mut store);
        {
            {
                let mut input_stream = store.get_resource_mut::<EventStream<InputEvent>>().unwrap();
                input_stream.publish(InputEvent {
                    input: Input::Keyboard { key: Key::D },
                    value: 1.0,
                });
            }
            AnySystem::run(&mut input_system, &store);
            let input_handler = store.get_resource::<InputHandler<UserInput>>().unwrap();
            assert_eq!(input_handler.axis_value(UserInput::X), Some(1.0));
        }

        AnySystem::dispose(&mut input_system, &mut store);
    }

    #[test]
    fn retrieve_axis_value_with_scale() {
        let (mut store, mut input_system) = setup_test();

        AnySystem::init(&mut input_system, &mut store);
        {
            {
                let mut input_stream = store.get_resource_mut::<EventStream<InputEvent>>().unwrap();
                input_stream.publish(InputEvent {
                    input: Input::Keyboard { key: Key::A },
                    value: 1.0,
                });
            }
            AnySystem::run(&mut input_system, &store);
            let input_handler = store.get_resource::<InputHandler<UserInput>>().unwrap();
            assert_eq!(input_handler.axis_value(UserInput::X), Some(-1.0));
        }

        AnySystem::dispose(&mut input_system, &mut store);
    }

    #[test]
    fn retrieve_axis_value_with_override() {
        let (mut store, mut input_system) = setup_test();

        AnySystem::init(&mut input_system, &mut store);
        {
            {
                let mut input_stream = store.get_resource_mut::<EventStream<InputEvent>>().unwrap();
                input_stream.publish(InputEvent {
                    input: Input::Keyboard { key: Key::A },
                    value: 1.0,
                });
                input_stream.publish(InputEvent {
                    input: Input::Keyboard { key: Key::D },
                    value: 1.0,
                });
            }
            AnySystem::run(&mut input_system, &store);
            let input_handler = store.get_resource::<InputHandler<UserInput>>().unwrap();
            assert_eq!(input_handler.axis_value(UserInput::X), Some(1.0));
        }

        AnySystem::dispose(&mut input_system, &mut store);
    }
}

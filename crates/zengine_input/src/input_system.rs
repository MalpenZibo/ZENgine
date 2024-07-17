use crate::{input::InputEvent, Bindings, InputHandler, InputType};
use zengine_ecs::system::{EventStream, ResMut};

pub(crate) fn input_system<T: InputType>(
    bindings: Bindings<T>,
) -> impl Fn(EventStream<InputEvent>, ResMut<InputHandler<T>>) {
    move |event_stream: EventStream<InputEvent>, mut input_handler: ResMut<InputHandler<T>>| {
        for e in event_stream.read() {
            if let Some(action_mappings) = &bindings.action_mappings {
                for actions in action_mappings.iter() {
                    if let Some(_action) = actions.1.iter().find(|action| action.source == e.input)
                    {
                        match input_handler.actions_value.get_mut(actions.0) {
                            Some(entry) => {
                                match entry.iter().position(|value| value.0 == e.input) {
                                    Some(index) => {
                                        if e.value > 0.0 {
                                            entry[index].1 = e.value > 0.0;
                                        } else {
                                            entry.remove(index);
                                        }
                                    }
                                    None => {
                                        if e.value > 0.0 {
                                            entry.push((e.input.clone(), e.value > 0.0))
                                        }
                                    }
                                }
                            }
                            None => {
                                if e.value > 0.0 {
                                    input_handler.actions_value.insert(
                                        actions.0.clone(),
                                        vec![(e.input.clone(), e.value > 0.0)],
                                    );
                                }
                            }
                        }
                    }
                }
            }
            if let Some(axis_mappings) = &bindings.axis_mappings {
                for axes in axis_mappings.iter() {
                    if let Some(axis) = axes.1.iter().find(|axis| axis.source == e.input) {
                        let value = if let Some(discrete_map) = axis.discrete_map {
                            if f32::abs(e.value) > f32::abs(discrete_map) {
                                if e.value < 0. {
                                    -1.
                                } else {
                                    1.
                                }
                            } else {
                                0.
                            }
                        } else {
                            e.value
                        } * if axis.invert { -1. } else { 1. };
                        match input_handler.axes_value.get_mut(axes.0) {
                            Some(entry) => {
                                match entry.iter().position(|value| value.0 == e.input) {
                                    Some(index) => {
                                        if e.value != 0.0 {
                                            entry[index].1 = value;
                                        } else {
                                            entry.remove(index);
                                        }
                                    }
                                    None => {
                                        if e.value != 0.0 {
                                            entry.push((e.input.clone(), value))
                                        }
                                    }
                                }
                            }
                            None => {
                                if e.value != 0.0 {
                                    input_handler
                                        .axes_value
                                        .insert(axes.0.clone(), vec![(e.input.clone(), value)]);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{input::Input, ActionBind, AxisBind};

    use super::*;
    use crate::device::Key;
    use serde::Deserialize;
    use zengine_ecs::{
        system::{IntoSystem, System},
        World,
    };

    #[derive(Eq, PartialEq, Hash, Deserialize, Clone, Default, Debug)]
    enum UserInput {
        #[default]
        Jump,
        X,
    }

    impl InputType for UserInput {}

    fn setup_test() -> (World, impl System) {
        let world = World::default();

        let content = "
            axis_mappings: 
                X:
                - source:
                    Keyboard:
                        key: D
                - source:
                    Keyboard:
                        key: A
                  invert: true
            action_mappings:
                Jump:
                - source: 
                    Keyboard: 
                        key: Space
        ";

        let bindings: Bindings<UserInput> = serde_yaml::from_str(content).unwrap();

        let input_system = IntoSystem::into_system(input_system(bindings));

        (world, input_system)
    }

    #[test]
    fn decode_typed_bindings_from_yaml() {
        let content = "
            axis_mappings: 
                X:
                - source:
                    Keyboard:
                        key: D
                - source:
                    Keyboard:
                        key: A
                  invert: true
            action_mappings:
                Jump:
                - source: 
                    Keyboard: 
                        key: Space
        ";

        let bindings: Bindings<UserInput> = serde_yaml::from_str(content).unwrap();

        assert_eq!(
            bindings
                .action_mappings
                .unwrap()
                .get(&UserInput::Jump)
                .unwrap(),
            &vec!(ActionBind {
                source: Input::Keyboard { key: Key::Space }
            })
        );

        assert_eq!(
            bindings.axis_mappings.unwrap().get(&UserInput::X).unwrap(),
            &vec!(
                AxisBind {
                    source: Input::Keyboard { key: Key::KeyD },
                    invert: false,
                    discrete_map: None
                },
                AxisBind {
                    source: Input::Keyboard { key: Key::KeyA },
                    invert: true,
                    discrete_map: None
                }
            )
        );
    }

    #[test]
    fn decode_generics_bindings_from_yaml() {
        let content = "
            axis_mappings: 
                X:
                - source:
                    Keyboard:
                        key: D
                - source:
                    Keyboard:
                        key: A
                  invert: true
            action_mappings:
                Jump:
                - source: 
                    Keyboard: 
                        key: Space
        ";

        let bindings: Bindings<String> = serde_yaml::from_str(content).unwrap();

        assert_eq!(
            bindings.action_mappings.unwrap().get("Jump").unwrap(),
            &vec!(ActionBind {
                source: Input::Keyboard { key: Key::Space }
            })
        );

        assert_eq!(
            bindings.axis_mappings.unwrap().get("X").unwrap(),
            &vec!(
                AxisBind {
                    source: Input::Keyboard { key: Key::KeyD },
                    invert: false,
                    discrete_map: None
                },
                AxisBind {
                    source: Input::Keyboard { key: Key::KeyA },
                    invert: true,
                    discrete_map: None
                }
            )
        );
    }

    #[test]
    fn retrieve_action_value() {
        let (mut world, mut input_system) = setup_test();

        input_system.init(&mut world);
        {
            let mut input_stream = world.get_mut_event_handler::<InputEvent>().unwrap();
            input_stream.publish(InputEvent {
                input: Input::Keyboard { key: Key::Space },
                value: 1.0,
            });
        }
        input_system.run(&world);

        let input_handler = world.get_resource::<InputHandler<UserInput>>().unwrap();
        assert!(input_handler.action_value(UserInput::Jump));
    }

    #[test]
    fn retrieve_axis_value() {
        let (mut world, mut input_system) = setup_test();

        input_system.init(&mut world);
        {
            let mut input_stream = world.get_mut_event_handler::<InputEvent>().unwrap();
            input_stream.publish(InputEvent {
                input: Input::Keyboard { key: Key::KeyD },
                value: 1.0,
            });
        }
        input_system.run(&world);

        let input_handler = world.get_resource::<InputHandler<UserInput>>().unwrap();
        assert_eq!(input_handler.axis_value(UserInput::X), 1.0);
    }

    #[test]
    fn retrieve_axis_value_with_scale() {
        let (mut world, mut input_system) = setup_test();

        input_system.init(&mut world);
        {
            let mut input_stream = world.get_mut_event_handler::<InputEvent>().unwrap();
            input_stream.publish(InputEvent {
                input: Input::Keyboard { key: Key::KeyA },
                value: 1.0,
            });
        }
        input_system.run(&world);

        let input_handler = world.get_resource::<InputHandler<UserInput>>().unwrap();
        assert_eq!(input_handler.axis_value(UserInput::X), -1.0);
    }

    #[test]
    fn retrieve_axis_value_with_override() {
        let (mut world, mut input_system) = setup_test();

        input_system.init(&mut world);
        {
            let mut input_stream = world.get_mut_event_handler::<InputEvent>().unwrap();
            input_stream.publish(InputEvent {
                input: Input::Keyboard { key: Key::KeyA },
                value: 1.0,
            });
            input_stream.publish(InputEvent {
                input: Input::Keyboard { key: Key::KeyD },
                value: 1.0,
            });
        }
        input_system.run(&world);

        let input_handler = world.get_resource::<InputHandler<UserInput>>().unwrap();
        assert_eq!(input_handler.axis_value(UserInput::X), 1.0);
    }
}

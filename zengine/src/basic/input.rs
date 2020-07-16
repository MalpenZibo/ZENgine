use crate::core::system::System;
use crate::event::event::Input;
use crate::event::keyboard::Key;
use fnv::FnvHashMap;
use serde::Deserialize;
use std::any::Any;
use std::hash::Hash;

pub trait InputType: Any + Eq + PartialEq + Hash {}

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
    pub axes: FnvHashMap<T, Vec<Axis>>,
    pub actions: FnvHashMap<T, Vec<Action>>,
}

#[derive(Default)]
pub struct InputSystem<T: InputType> {
    bindings: Bindings<T>,
}

impl<T: InputType> InputSystem<T> {
    pub fn from_bindings(bindings: Bindings<T>) -> Self {
        InputSystem { bindings: bindings }
    }
}

impl<'a, T: InputType> System<'a> for InputSystem<T> {
    type Data = ();

    fn run(&mut self, data: Self::Data) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Eq, PartialEq, Hash, Deserialize)]
    enum UserInput {
        Jump,
        X,
    }

    impl InputType for UserInput {}

    #[test]
    fn decode_typed_bindings_from_yaml() {
        let content = "
            axes: 
                X:
                - source:
                    Keyboard:
                        key: D
                  scale: 1.0
                - source:
                    Keyboard:
                        key: A
                  scale: -1.0
            actions:
                Jump:
                - source: 
                    Keyboard: 
                        key: Space
        ";

        let bindings: Bindings<UserInput> = serde_yaml::from_str(&content).unwrap();

        assert_eq!(
            bindings.actions.get(&UserInput::Jump).unwrap(),
            &vec!(Action {
                source: Input::Keyboard { key: Key::Space }
            })
        );

        assert_eq!(
            bindings.axes.get(&UserInput::X).unwrap(),
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
            axes: 
                X:
                - source:
                    Keyboard:
                        key: D
                  scale: 1.0
                - source:
                    Keyboard:
                        key: A
                  scale: -1.0
            actions:
                Jump:
                - source: 
                    Keyboard: 
                        key: Space
        ";

        let bindings: Bindings<String> = serde_yaml::from_str(&content).unwrap();

        assert_eq!(
            bindings.actions.get("Jump").unwrap(),
            &vec!(Action {
                source: Input::Keyboard { key: Key::Space }
            })
        );

        assert_eq!(
            bindings.axes.get("X").unwrap(),
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
/*

{
  "axis": {
    "move_y": [
       { "source": "W", "scale": 1.0 },
       { "source": "S", "scale": -1.0 },
       { "source": "Controller1.LeftStick.Y", "scale": 1.0 }
    ],
    "move_x": [
       { "source": "D", "scale": 1.0 },
       { "source": "A", "scale": -1.0 },
       { "source": "Controller1.LeftStick.X", "scale": 1.0 }
    ]
  },
  "actions": {
    "jump": [
       { "source": "SPACEBAR", "mod": "(something like shift, ctrl, alt, etc..." },
       { "source": "Controller1.Button.A" }
    ]
  }
}

*/

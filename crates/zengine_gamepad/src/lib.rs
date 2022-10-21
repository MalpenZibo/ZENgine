use gilrs::Gilrs;
use std::ops::{Deref, DerefMut};
use zengine_engine::Module;
use zengine_macro::UnsendableResource;

/// Adds gamepad support to the engine
///
/// NB: currently the gamepad support is not provided for Android platform
pub struct GamepadModule;

#[derive(UnsendableResource, Debug)]
struct GamepadHandler(Gilrs);

impl Deref for GamepadHandler {
    type Target = Gilrs;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for GamepadHandler {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Module for GamepadModule {
    #[cfg(not(target_os = "android"))]
    fn init(self, engine: &mut zengine_engine::Engine) {
        let gilrs = Gilrs::new().unwrap();

        engine
            .world
            .create_unsendable_resource(GamepadHandler(gilrs));
        engine.add_system_into_stage(gamepad_system, zengine_engine::Stage::PreUpdate);
    }

    #[cfg(target_os = "android")]
    fn init(self, _engine: &mut zengine_engine::Engine) {
        log::warn!("Gamepad not supported on android")
    }
}

#[cfg(not(target_os = "android"))]
fn gamepad_system(
    gamepad_handler: Option<zengine_ecs::system::UnsendableResMut<GamepadHandler>>,
    mut input: zengine_ecs::system::EventPublisher<zengine_input::InputEvent>,
) {
    use zengine_input::{
        device::{ControllerButton, Which},
        Axis, Input, InputEvent,
    };

    if let Some(mut gamepad_handler) = gamepad_handler {
        while let Some(gilrs::Event { id, event, .. }) = gamepad_handler.next_event() {
            match event {
                gilrs::EventType::ButtonPressed(button, ..) => input.publish(InputEvent {
                    input: Input::ControllerButton {
                        device_id: id.into(),
                        button,
                    },
                    value: 1.0,
                }),
                gilrs::EventType::ButtonReleased(button, ..) => input.publish(InputEvent {
                    input: Input::ControllerButton {
                        device_id: id.into(),
                        button,
                    },
                    value: 0.0,
                }),
                gilrs::EventType::AxisChanged(axis, value, ..) => match axis {
                    gilrs::Axis::LeftStickX => input.publish(InputEvent {
                        input: Input::ControllerStick {
                            device_id: id.into(),
                            which: Which::Left,
                            axis: Axis::X,
                        },
                        value,
                    }),
                    gilrs::Axis::LeftStickY => input.publish(InputEvent {
                        input: Input::ControllerStick {
                            device_id: id.into(),
                            which: Which::Left,
                            axis: Axis::Y,
                        },
                        value,
                    }),
                    gilrs::Axis::RightStickX => input.publish(InputEvent {
                        input: Input::ControllerStick {
                            device_id: id.into(),
                            which: Which::Right,
                            axis: Axis::X,
                        },
                        value,
                    }),
                    gilrs::Axis::RightStickY => input.publish(InputEvent {
                        input: Input::ControllerStick {
                            device_id: id.into(),
                            which: Which::Right,
                            axis: Axis::Y,
                        },
                        value,
                    }),
                    gilrs::Axis::LeftZ => input.publish(InputEvent {
                        input: Input::ControllerTrigger {
                            device_id: id.into(),
                            which: Which::Left,
                        },
                        value,
                    }),
                    gilrs::Axis::RightZ => input.publish(InputEvent {
                        input: Input::ControllerTrigger {
                            device_id: id.into(),
                            which: Which::Right,
                        },
                        value,
                    }),
                    gilrs::Axis::DPadX => input.publish(InputEvent {
                        input: Input::ControllerButton {
                            device_id: id.into(),
                            button: if value < 0.0 {
                                ControllerButton::DPadLeft
                            } else {
                                ControllerButton::DPadRight
                            },
                        },
                        value,
                    }),
                    gilrs::Axis::DPadY => input.publish(InputEvent {
                        input: Input::ControllerButton {
                            device_id: id.into(),
                            button: if value < 0.0 {
                                ControllerButton::DPadDown
                            } else {
                                ControllerButton::DPadUp
                            },
                        },
                        value,
                    }),
                    _ => {}
                },

                _ => {}
            }
        }
    }
}

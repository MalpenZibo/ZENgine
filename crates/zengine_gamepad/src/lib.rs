use gilrs::Gilrs;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};
use zengine_engine::Module;
use zengine_input::device::ControllerButton;
use zengine_macro::UnsendableResource;

/// Adds gamepad support to the engine
///
/// NB: currently the gamepad support is not provided for Android platform
#[derive(Debug)]
pub struct GamepadModule(pub Option<HashMap<u32, ControllerButton>>);

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
        gilrs.gamepads().for_each(|(id, gamepad)| {
            log::info!("Found gamepad: {} - {}", id, gamepad.name());
        });

        engine
            .world
            .create_unsendable_resource(GamepadHandler(gilrs));
        engine.add_system_into_stage(gamepad_system(self.0), zengine_engine::Stage::PreUpdate);
    }

    #[cfg(target_os = "android")]
    fn init(self, _engine: &mut zengine_engine::Engine) {
        log::warn!("Gamepad not supported on android")
    }
}

#[cfg(not(target_os = "android"))]
fn gamepad_system(
    mapping: Option<HashMap<u32, ControllerButton>>,
) -> impl Fn(
    Option<zengine_ecs::system::UnsendableResMut<GamepadHandler>>,
    zengine_ecs::system::EventPublisher<zengine_input::InputEvent>,
) {
    use gilrs::Button;
    use log::debug;
    use zengine_input::{
        device::{ControllerButton, Which},
        Axis, Input, InputEvent,
    };

    move |gamepad_handler, mut input| {
        if let Some(mut gamepad_handler) = gamepad_handler {
            while let Some(gilrs::Event { id, event, .. }) = gamepad_handler.next_event() {
                debug!("Gamepad event: {:?}", event);
                match event {
                    gilrs::EventType::ButtonPressed(Button::Unknown, code, ..) => {
                        if let Some(mapping) = &mapping {
                            if let Some(button) = mapping.get(&code.into_u32()) {
                                input.publish(InputEvent {
                                    input: Input::ControllerButton {
                                        device_id: id.into(),
                                        button: *button,
                                    },
                                    value: 1.0,
                                });
                            }
                        }
                    }
                    gilrs::EventType::ButtonReleased(Button::Unknown, code, ..) => {
                        if let Some(mapping) = &mapping {
                            if let Some(button) = mapping.get(&code.into_u32()) {
                                input.publish(InputEvent {
                                    input: Input::ControllerButton {
                                        device_id: id.into(),
                                        button: *button,
                                    },
                                    value: 0.0,
                                });
                            }
                        }
                    }
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
}

use log::info;
use sdl2::event::Event;
use zengine_ecs::system_parameter::{
    EventPublisher, OptionalUnsendableRes, OptionalUnsendableResMut,
};
use zengine_engine::EngineEvent;
use zengine_platform::{
    device::{
        controller::{ControllerButton, Which},
        keyboard::Key,
        mouse::MouseButton,
    },
    Controllers, PlatformContext,
};

use crate::input::{Axis, Input, InputEvent};

pub fn event_system(
    context: OptionalUnsendableResMut<PlatformContext>,
    controllers: OptionalUnsendableRes<Controllers>,
    mut engine_event: EventPublisher<EngineEvent>,
    mut input: EventPublisher<InputEvent>,
) {
    let controllers = &controllers.expect("controllets not present").0;

    for event in context
        .expect("sdl context not present")
        .event_pump
        .poll_iter()
    {
        match event {
            Event::Quit { .. } => {
                info!("quit event sended");
                engine_event.publish(EngineEvent::Quit);
            }
            Event::KeyDown {
                keycode: Some(keycode),
                ..
            } => input.publish(InputEvent {
                input: Input::Keyboard {
                    key: Key::from_sdl2_key(keycode),
                },
                value: 1.0,
            }),
            Event::KeyUp {
                keycode: Some(keycode),
                ..
            } => input.publish(InputEvent {
                input: Input::Keyboard {
                    key: Key::from_sdl2_key(keycode),
                },
                value: 0.0,
            }),
            Event::MouseMotion { x, y, .. } => {
                input.publish(InputEvent {
                    input: Input::MouseMotion { axis: Axis::X },
                    value: x as f32,
                });
                input.publish(InputEvent {
                    input: Input::MouseMotion { axis: Axis::Y },
                    value: y as f32,
                });
            }
            Event::MouseWheel { x, y, .. } => {
                input.publish(InputEvent {
                    input: Input::MouseWheel { axis: Axis::X },
                    value: x as f32,
                });
                input.publish(InputEvent {
                    input: Input::MouseWheel { axis: Axis::Y },
                    value: y as f32,
                });
            }
            Event::MouseButtonDown { mouse_btn, .. } => input.publish(InputEvent {
                input: Input::MouseButton {
                    button: MouseButton::from_sdl_button(mouse_btn),
                },
                value: 1.0,
            }),
            Event::MouseButtonUp { mouse_btn, .. } => input.publish(InputEvent {
                input: Input::MouseButton {
                    button: MouseButton::from_sdl_button(mouse_btn),
                },
                value: 0.0,
            }),

            Event::ControllerButtonDown { which, button, .. } => {
                if let Some(c) = controllers.get(&which) {
                    input.publish(InputEvent {
                        input: Input::ControllerButton {
                            device_id: c.0,
                            button: ControllerButton::from_sdl_button(button),
                        },
                        value: 1.0,
                    })
                }
            }
            Event::ControllerButtonUp { which, button, .. } => {
                if let Some(c) = controllers.get(&which) {
                    input.publish(InputEvent {
                        input: Input::ControllerButton {
                            device_id: c.0,
                            button: ControllerButton::from_sdl_button(button),
                        },
                        value: 0.0,
                    })
                }
            }
            Event::ControllerAxisMotion {
                which, axis, value, ..
            } => {
                if let Some(c) = controllers.get(&which) {
                    input.publish(InputEvent {
                        input: match axis {
                            sdl2::controller::Axis::LeftX => Input::ControllerStick {
                                device_id: c.0,
                                which: Which::Left,
                                axis: Axis::X,
                            },
                            sdl2::controller::Axis::LeftY => Input::ControllerStick {
                                device_id: c.0,
                                which: Which::Left,
                                axis: Axis::Y,
                            },
                            sdl2::controller::Axis::RightX => Input::ControllerStick {
                                device_id: c.0,
                                which: Which::Right,
                                axis: Axis::X,
                            },
                            sdl2::controller::Axis::RightY => Input::ControllerStick {
                                device_id: c.0,
                                which: Which::Right,
                                axis: Axis::Y,
                            },
                            sdl2::controller::Axis::TriggerLeft => Input::ControllerTrigger {
                                device_id: c.0,
                                which: Which::Left,
                            },
                            sdl2::controller::Axis::TriggerRight => Input::ControllerTrigger {
                                device_id: c.0,
                                which: Which::Right,
                            },
                        },
                        value: {
                            if value > -8000i16 && value < 8000i16 {
                                0.0
                            } else if (value).is_positive() {
                                (value) as f32 / std::i16::MAX as f32
                            } else {
                                ((value) as f32).abs() / std::i16::MIN as f32
                            }
                        },
                    })
                }
            }
            _ => {}
        }
    }
}

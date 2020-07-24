extern crate sdl2;

use crate::core::system::System;
use crate::core::system::Write;
use crate::core::Store;
use crate::core::Trans;
use crate::event::controller::ControllerButton;
use crate::event::controller::Which;
use crate::event::event::InputEvent;
use crate::event::event::{Axis, Input};
use crate::event::event_stream::EventStream;
use crate::event::keyboard::Key;
use crate::event::mouse::MouseButton;
use log::{info, trace};
use sdl2::event::Event;
use sdl2::video::GLProfile;
use sdl2::EventPump;
use sdl2::Sdl;

pub struct PlatformSystem {
    sdl_context: Sdl,
    event_pump: EventPump,
}

impl Default for PlatformSystem {
    fn default() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();
        let controller_subsystem = sdl_context.game_controller().unwrap();

        let available = controller_subsystem
            .num_joysticks()
            .map_err(|e| format!("can't enumerate joysticks: {}", e))
            .unwrap();

        info!("{} joysticks available", available);

        // Iterate over all available joysticks and look for game controllers.
        for id in 0..available {
            if !controller_subsystem.is_game_controller(id) {
                trace!("{} is not a game controller", id);
            }

            trace!("Attempting to open controller {}", id);

            match controller_subsystem.open(id) {
                Ok(c) => {
                    // We managed to find and open a game controller,
                    // exit the loop
                    trace!("Success: opened \"{}\"", c.name());
                }
                Err(e) => {
                    trace!("failed: {:?}", e);
                }
            }
        }
        PlatformSystem {
            sdl_context: sdl_context,
            event_pump: event_pump,
        }
    }
}

impl<'a> System<'a> for PlatformSystem {
    type Data = (
        Write<'a, EventStream<Trans>>,
        Write<'a, EventStream<InputEvent>>,
    );

    fn init(&mut self, store: &mut Store) {
        let video_subsystem = self.sdl_context.video().unwrap();

        store.insert_resource(video_subsystem);
    }

    fn run(&mut self, (mut trans, mut input): Self::Data) {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    info!("quit event sended");
                    trans.publish(Trans::Quit);
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => input.publish(InputEvent {
                    input: Input::Keyboard {
                        key: Key::from_sdl2_key(&keycode),
                    },
                    value: 1.0,
                }),
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => input.publish(InputEvent {
                    input: Input::Keyboard {
                        key: Key::from_sdl2_key(&keycode),
                    },
                    value: 0.0,
                }),
                Event::MouseMotion { x, y, .. } => {
                    input.publish(InputEvent {
                        input: Input::MouseMotion { axis: Axis::X },
                        value: if x > -8000i32 && x < 8000i32 {
                            0.0
                        } else {
                            if (x).is_positive() {
                                (x) as f32 / std::i32::MAX as f32
                            } else {
                                ((x) as f32).abs() / std::i32::MIN as f32
                            }
                        },
                    });
                    input.publish(InputEvent {
                        input: Input::MouseMotion { axis: Axis::Y },
                        value: if y > -8000i32 && y < 8000i32 {
                            0.0
                        } else {
                            if (y).is_positive() {
                                (y) as f32 / std::i32::MAX as f32
                            } else {
                                ((y) as f32).abs() / std::i32::MIN as f32
                            }
                        },
                    });
                }
                Event::MouseWheel { x, y, .. } => {
                    input.publish(InputEvent {
                        input: Input::MouseWheel { axis: Axis::X },
                        value: if x > -8000i32 && x < 8000i32 {
                            0.0
                        } else {
                            if (x).is_positive() {
                                (x) as f32 / std::i32::MAX as f32
                            } else {
                                ((x) as f32).abs() / std::i32::MIN as f32
                            }
                        },
                    });
                    input.publish(InputEvent {
                        input: Input::MouseWheel { axis: Axis::Y },
                        value: if y > -8000i32 && y < 8000i32 {
                            0.0
                        } else {
                            if (y).is_positive() {
                                (y) as f32 / std::i32::MAX as f32
                            } else {
                                ((y) as f32).abs() / std::i32::MIN as f32
                            }
                        },
                    });
                }
                Event::MouseButtonDown { mouse_btn, .. } => input.publish(InputEvent {
                    input: Input::MouseButton {
                        button: MouseButton::from_sdl_button(&mouse_btn),
                    },
                    value: 1.0,
                }),
                Event::MouseButtonUp { mouse_btn, .. } => input.publish(InputEvent {
                    input: Input::MouseButton {
                        button: MouseButton::from_sdl_button(&mouse_btn),
                    },
                    value: 0.0,
                }),

                Event::ControllerButtonDown { which, button, .. } => input.publish(InputEvent {
                    input: Input::ControllerButton {
                        device_id: which,
                        button: ControllerButton::from_sdl_button(&button),
                    },
                    value: 1.0,
                }),
                Event::ControllerButtonUp { which, button, .. } => input.publish(InputEvent {
                    input: Input::ControllerButton {
                        device_id: which,
                        button: ControllerButton::from_sdl_button(&button),
                    },
                    value: 0.0,
                }),

                Event::ControllerAxisMotion {
                    which, axis, value, ..
                } => input.publish(InputEvent {
                    input: match axis {
                        sdl2::controller::Axis::LeftX => Input::ControllerStick {
                            device_id: which,
                            which: Which::Left,
                            axis: Axis::X,
                        },
                        sdl2::controller::Axis::LeftY => Input::ControllerStick {
                            device_id: which,
                            which: Which::Left,
                            axis: Axis::Y,
                        },
                        sdl2::controller::Axis::RightX => Input::ControllerStick {
                            device_id: which,
                            which: Which::Right,
                            axis: Axis::X,
                        },
                        sdl2::controller::Axis::RightY => Input::ControllerStick {
                            device_id: which,
                            which: Which::Right,
                            axis: Axis::Y,
                        },
                        sdl2::controller::Axis::TriggerLeft => Input::ControllerTrigger {
                            device_id: which,
                            which: Which::Left,
                        },
                        sdl2::controller::Axis::TriggerRight => Input::ControllerTrigger {
                            device_id: which,
                            which: Which::Right,
                        },
                    },
                    value: {
                        if value > -8000i16 && value < 8000i16 {
                            0.0
                        } else {
                            if (value).is_positive() {
                                (value) as f32 / std::i16::MAX as f32
                            } else {
                                ((value) as f32).abs() / std::i16::MIN as f32
                            }
                        }
                    },
                }),
                _ => {}
            }
        }
    }
}

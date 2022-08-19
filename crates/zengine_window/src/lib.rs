use std::ops::Deref;

use gilrs::Gilrs;
use glutin::{
    dpi::LogicalSize,
    event::{ElementState, Event, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoopWindowTarget},
    window::WindowBuilder,
    Api, GlProfile, GlRequest,
};
use glutin::{ContextBuilder, ContextWrapper, PossiblyCurrent};
use zengine_ecs::UnsendableResource;
use zengine_engine::{Engine, Module};
use zengine_input::{
    device::{ControllerButton, Which},
    Axis, Input, InputEvent,
};

#[derive(Debug, Clone)]
pub struct WindowSpecs {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
}

impl WindowSpecs {
    pub fn new(title: String, width: u32, height: u32, fullscreen: bool) -> Self {
        WindowSpecs {
            title,
            width,
            height,
            fullscreen,
        }
    }
}

impl Default for WindowSpecs {
    fn default() -> Self {
        WindowSpecs {
            title: String::from("zengine"),
            width: 800,
            height: 600,
            fullscreen: false,
        }
    }
}

#[derive(Debug)]
pub struct Window(ContextWrapper<PossiblyCurrent, glutin::window::Window>);

impl UnsendableResource for Window {}

impl Deref for Window {
    type Target = ContextWrapper<PossiblyCurrent, glutin::window::Window>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
struct EventLoop(glutin::event_loop::EventLoop<()>);

impl UnsendableResource for EventLoop {}

#[derive(Debug)]
pub struct WindowModule(pub WindowSpecs);
impl Module for WindowModule {
    fn init(self, engine: &mut Engine) {
        let event_loop = glutin::event_loop::EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(self.0.title)
            .with_inner_size(LogicalSize::new(self.0.width, self.0.height));
        let mut gl_window = ContextBuilder::new()
            .with_double_buffer(Some(true))
            .with_gl_profile(GlProfile::Core)
            .with_vsync(true);
        if cfg!(target_os = "macos") {
            gl_window = gl_window.with_gl(GlRequest::Specific(Api::OpenGl, (4, 1)));
        } else {
            gl_window = gl_window.with_gl(GlRequest::Specific(Api::OpenGl, (4, 6)));
        }

        let gl_window = gl_window.build_windowed(window, &event_loop).unwrap();
        let gl_window = unsafe { gl_window.make_current() }.unwrap();

        engine.world.create_unsendable_resource(Window(gl_window));
        engine
            .world
            .create_unsendable_resource(EventLoop(event_loop));

        engine.set_runner(runner);
    }
}

fn runner(mut engine: Engine) {
    let event_loop = engine
        .world
        .remove_unsendable_resource::<EventLoop>()
        .unwrap();

    let mut gilrs = Gilrs::new().unwrap();

    let event_handler = move |event: Event<()>,
                              _event_loop: &EventLoopWindowTarget<()>,
                              control_flow: &mut ControlFlow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => {
                let mut input = engine.world.get_mut_event_handler::<InputEvent>().unwrap();
                input.publish(InputEvent {
                    input: Input::MouseButton { button },
                    value: if state == ElementState::Pressed {
                        1.0
                    } else {
                        0.0
                    },
                })
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                let mut input = engine.world.get_mut_event_handler::<InputEvent>().unwrap();
                input.publish(InputEvent {
                    input: Input::MouseMotion { axis: Axis::X },
                    value: position.x as f32,
                });
                input.publish(InputEvent {
                    input: Input::MouseMotion { axis: Axis::Y },
                    value: position.y as f32,
                });
            }
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => match delta {
                MouseScrollDelta::LineDelta(x, y) => {
                    let mut input = engine.world.get_mut_event_handler::<InputEvent>().unwrap();
                    input.publish(InputEvent {
                        input: Input::MouseWheel { axis: Axis::X },
                        value: x as f32,
                    });
                    input.publish(InputEvent {
                        input: Input::MouseWheel { axis: Axis::Y },
                        value: y as f32,
                    });
                }
                MouseScrollDelta::PixelDelta(p) => {
                    let mut input = engine.world.get_mut_event_handler::<InputEvent>().unwrap();
                    input.publish(InputEvent {
                        input: Input::MouseWheel { axis: Axis::X },
                        value: p.x as f32,
                    });
                    input.publish(InputEvent {
                        input: Input::MouseWheel { axis: Axis::Y },
                        value: p.y as f32,
                    });
                }
            },
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input: keyboard_input,
                        ..
                    },
                ..
            } => {
                let mut input = engine.world.get_mut_event_handler::<InputEvent>().unwrap();
                input.publish(InputEvent {
                    input: Input::Keyboard {
                        key: keyboard_input.virtual_keycode.unwrap(),
                    },
                    value: if keyboard_input.state == ElementState::Pressed {
                        1.0
                    } else {
                        0.0
                    },
                })
            }
            Event::MainEventsCleared => {
                {
                    let mut input = engine.world.get_mut_event_handler::<InputEvent>().unwrap();
                    while let Some(gilrs::Event { id, event, .. }) = gilrs.next_event() {
                        match event {
                            gilrs::EventType::ButtonPressed(button, ..) => {
                                input.publish(InputEvent {
                                    input: Input::ControllerButton {
                                        device_id: id,
                                        button,
                                    },
                                    value: 1.0,
                                })
                            }
                            gilrs::EventType::ButtonReleased(button, ..) => {
                                input.publish(InputEvent {
                                    input: Input::ControllerButton {
                                        device_id: id,
                                        button,
                                    },
                                    value: 0.0,
                                })
                            }
                            gilrs::EventType::AxisChanged(axis, value, ..) => match axis {
                                gilrs::Axis::LeftStickX => input.publish(InputEvent {
                                    input: Input::ControllerStick {
                                        device_id: id,
                                        which: Which::Left,
                                        axis: Axis::X,
                                    },
                                    value,
                                }),
                                gilrs::Axis::LeftStickY => input.publish(InputEvent {
                                    input: Input::ControllerStick {
                                        device_id: id,
                                        which: Which::Left,
                                        axis: Axis::Y,
                                    },
                                    value,
                                }),
                                gilrs::Axis::RightStickX => input.publish(InputEvent {
                                    input: Input::ControllerStick {
                                        device_id: id,
                                        which: Which::Right,
                                        axis: Axis::X,
                                    },
                                    value,
                                }),
                                gilrs::Axis::RightStickY => input.publish(InputEvent {
                                    input: Input::ControllerStick {
                                        device_id: id,
                                        which: Which::Right,
                                        axis: Axis::Y,
                                    },
                                    value,
                                }),
                                gilrs::Axis::LeftZ => input.publish(InputEvent {
                                    input: Input::ControllerTrigger {
                                        device_id: id,
                                        which: Which::Left,
                                    },
                                    value,
                                }),
                                gilrs::Axis::RightZ => input.publish(InputEvent {
                                    input: Input::ControllerTrigger {
                                        device_id: id,
                                        which: Which::Right,
                                    },
                                    value,
                                }),
                                gilrs::Axis::DPadX => input.publish(InputEvent {
                                    input: Input::ControllerButton {
                                        device_id: id,
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
                                        device_id: id,
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

                if engine.update() {
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => (),
        }
    };

    event_loop.0.run(event_handler);
}

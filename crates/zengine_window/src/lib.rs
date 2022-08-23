use std::ops::Deref;

use gilrs::Gilrs;
use log::info;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoopWindowTarget},
    window::{Fullscreen, WindowBuilder},
};
use zengine_ecs::UnsendableResource;
use zengine_engine::{Engine, Module};
use zengine_input::{
    device::{ControllerButton, Which},
    Axis, Input, InputEvent,
};
use zengine_macro::Resource;

#[derive(Resource, Debug, Clone)]
pub struct WindowSpecs {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    pub vsync: bool,
}

impl Default for WindowSpecs {
    fn default() -> Self {
        WindowSpecs {
            title: String::from("zengine"),
            width: 800,
            height: 600,
            fullscreen: false,
            vsync: false,
        }
    }
}

#[derive(Debug)]
pub struct Window(winit::window::Window);

impl UnsendableResource for Window {}

impl Deref for Window {
    type Target = winit::window::Window;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
struct EventLoop(winit::event_loop::EventLoop<()>);

impl UnsendableResource for EventLoop {}

#[derive(Debug)]
pub struct WindowModule(pub WindowSpecs);
impl Module for WindowModule {
    fn init(self, engine: &mut Engine) {
        let event_loop = winit::event_loop::EventLoop::new();
        let mut window_builder = WindowBuilder::new()
            .with_title(self.0.title.clone())
            .with_inner_size(LogicalSize::new(self.0.width, self.0.height));

        if self.0.fullscreen {
            window_builder = window_builder
                .with_fullscreen(Some(Fullscreen::Borderless(event_loop.primary_monitor())));
        }

        let window = window_builder.build(&event_loop).unwrap();
        let size = window.inner_size();
        info!("size: {:?}", size);

        #[cfg(target_arch = "wasm32")]
        {
            // Winit prevents sizing with CSS, so we have to set
            // the size manually when on web.
            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("zengine-root")?;
                    let canvas = web_sys::Element::from(window.canvas());
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }

        engine.world.create_resource(self.0);
        engine.world.create_unsendable_resource(Window(window));
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

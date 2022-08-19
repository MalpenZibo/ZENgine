use std::ops::Deref;

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
use zengine_input::{Axis, Input, InputEvent};

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
        .extract_unsendable_resource::<EventLoop>()
        .unwrap();

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
                if engine.update() {
                    *control_flow = ControlFlow::Exit;
                }
            }

            // Event::ControllerButtonDown { which, button, .. } => {
            //     if let Some(c) = controllers.get(&which) {
            //         input.publish(InputEvent {
            //             input: Input::ControllerButton {
            //                 device_id: c.0,
            //                 button: ControllerButton::from_sdl_button(button),
            //             },
            //             value: 1.0,
            //         })
            //     }
            // }
            // Event::ControllerButtonUp { which, button, .. } => {
            //     if let Some(c) = controllers.get(&which) {
            //         input.publish(InputEvent {
            //             input: Input::ControllerButton {
            //                 device_id: c.0,
            //                 button: ControllerButton::from_sdl_button(button),
            //             },
            //             value: 0.0,
            //         })
            //     }
            // }
            // Event::ControllerAxisMotion {
            //     which, axis, value, ..
            // } => {
            //     if let Some(c) = controllers.get(&which) {
            //         input.publish(InputEvent {
            //             input: match axis {
            //                 sdl2::controller::Axis::LeftX => Input::ControllerStick {
            //                     device_id: c.0,
            //                     which: Which::Left,
            //                     axis: Axis::X,
            //                 },
            //                 sdl2::controller::Axis::LeftY => Input::ControllerStick {
            //                     device_id: c.0,
            //                     which: Which::Left,
            //                     axis: Axis::Y,
            //                 },
            //                 sdl2::controller::Axis::RightX => Input::ControllerStick {
            //                     device_id: c.0,
            //                     which: Which::Right,
            //                     axis: Axis::X,
            //                 },
            //                 sdl2::controller::Axis::RightY => Input::ControllerStick {
            //                     device_id: c.0,
            //                     which: Which::Right,
            //                     axis: Axis::Y,
            //                 },
            //                 sdl2::controller::Axis::TriggerLeft => Input::ControllerTrigger {
            //                     device_id: c.0,
            //                     which: Which::Left,
            //                 },
            //                 sdl2::controller::Axis::TriggerRight => Input::ControllerTrigger {
            //                     device_id: c.0,
            //                     which: Which::Right,
            //                 },
            //             },
            //             value: {
            //                 if value > -8000i16 && value < 8000i16 {
            //                     0.0
            //                 } else if (value).is_positive() {
            //                     (value) as f32 / std::i16::MAX as f32
            //                 } else {
            //                     ((value) as f32).abs() / std::i16::MIN as f32
            //                 }
            //             },
            //         })
            //     }
            // }
            _ => (),
        }
    };

    event_loop.0.run(event_handler);
}

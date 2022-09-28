use glam::UVec2;
use log::info;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoopWindowTarget},
    window::{Fullscreen, WindowBuilder},
};
use zengine_engine::{Engine, Module};
use zengine_input::{Axis, Input, InputEvent};
use zengine_macro::{Resource, UnsendableResource};

#[derive(Resource, Debug, Clone)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    pub vsync: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: String::from("zengine"),
            width: 800,
            height: 600,
            fullscreen: false,
            vsync: false,
        }
    }
}

#[derive(UnsendableResource, Debug)]
pub struct Window {
    pub internal: winit::window::Window,
}

#[derive(Resource, Default, Debug)]
pub struct WindowSpecs {
    pub size: UVec2,
    pub ratio: f32,
}

#[derive(UnsendableResource, Debug)]
struct EventLoop(winit::event_loop::EventLoop<()>);

#[derive(Default, Debug)]
pub struct WindowModule(pub WindowConfig);
impl Module for WindowModule {
    fn init(self, engine: &mut Engine) {
        let event_loop = winit::event_loop::EventLoop::new();
        let mut window_builder = WindowBuilder::new()
            .with_title(self.0.title.clone())
            .with_inner_size(LogicalSize::new(self.0.width, self.0.height))
            .with_resizable(false);

        if self.0.fullscreen {
            window_builder = window_builder
                .with_decorations(false)
                .with_fullscreen(Some(Fullscreen::Borderless(None)));
        }

        let window = window_builder.build(&event_loop).unwrap();
        let window_size = if self.0.fullscreen {
            let size = window.current_monitor().unwrap().size();

            (size.width, size.height)
        } else {
            window.inner_size().into()
        };
        info!("size: {:?}", window_size);

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
        engine
            .world
            .create_unsendable_resource(Window { internal: window });
        engine.world.create_resource(WindowSpecs {
            size: UVec2::new(window_size.0, window_size.1),
            ratio: window_size.0 as f32 / window_size.1 as f32,
        });
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

    let mut initialized = false;
    let mut window_with_size = false;

    let event_handler = move |event: Event<()>,
                              _event_loop: &EventLoopWindowTarget<()>,
                              control_flow: &mut ControlFlow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::Resumed => {
                initialized = true;

                if initialized && window_with_size {
                    engine.startup();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                {
                    let mut window_specs = engine.world.get_mut_resource::<WindowSpecs>().unwrap();
                    window_specs.size = UVec2::new(size.width, size.height);
                    window_specs.ratio = size.width as f32 / size.height as f32;
                }

                info!("New window size {:?}", size);

                window_with_size = true;

                if initialized && window_with_size {
                    engine.startup();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } if initialized && window_with_size => {
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
            } if initialized && window_with_size => {
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
            } if initialized && window_with_size => match delta {
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
            } if initialized && window_with_size => {
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
            Event::MainEventsCleared if initialized && window_with_size => {
                if engine.update() {
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => (),
        }
    };

    event_loop.0.run(event_handler);
}

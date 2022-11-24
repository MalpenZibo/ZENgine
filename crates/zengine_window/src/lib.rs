use glam::UVec2;
use log::info;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, MouseScrollDelta, TouchPhase, WindowEvent},
    event_loop::{ControlFlow, EventLoopWindowTarget},
    window::{Fullscreen, WindowBuilder},
};
use zengine_engine::{Engine, EngineEvent, Module};
use zengine_input::{Axis, Input, InputEvent};
use zengine_macro::{Resource, UnsendableResource};

#[cfg(target_os = "android")]
mod android_utils;

/// A [Resource](zengine_ecs::Resource) that defines the window configuration
#[derive(Resource, Debug, Clone)]
pub struct WindowConfig {
    /// Title of the window
    pub title: String,
    /// Width of the window
    pub width: u32,
    /// Height of the window
    pub height: u32,
    /// Flag to activate the fullscreen
    pub fullscreen: bool,
    /// Flag to activate the vsync
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

#[doc(hidden)]
#[derive(UnsendableResource, Debug)]
pub struct Window {
    pub internal: winit::window::Window,
}

/// A [Resource](zengine_ecs::Resource) that contains the active Windows settings
#[derive(Resource, Default, Debug)]
pub struct WindowSpecs {
    pub size: UVec2,
    pub ratio: f32,
    pub surface_id: usize,
}

#[derive(UnsendableResource, Debug)]
struct EventLoop(winit::event_loop::EventLoop<()>);

#[derive(Eq, PartialEq)]
enum RunnerState {
    Initializing { app_ready: bool, window_ready: bool },
    Running,
    Suspending,
    Suspended,
}

impl RunnerState {
    pub fn is_running(&self) -> bool {
        matches!(self, RunnerState::Running | RunnerState::Suspending)
    }

    pub fn can_start(&self) -> bool {
        matches!(
            self,
            RunnerState::Initializing {
                app_ready: true,
                window_ready: true
            }
        )
    }

    pub fn set_app_ready(&mut self) {
        if let RunnerState::Initializing { app_ready, .. } = self {
            *app_ready = true
        }
    }

    pub fn set_window_ready(&mut self) {
        if let RunnerState::Initializing { window_ready, .. } = self {
            *window_ready = true
        }
    }
}

///A [Module] that defines an interface for windowing support in ZENgine.
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

        #[cfg(target_os = "android")]
        {
            let result = android_utils::set_immersive_mode();
            if let Err(error) = result {
                log::warn!("Impossible to set the Android immersive mode: {}", error);
            }
        }

        engine.world.create_resource(self.0);
        engine
            .world
            .create_unsendable_resource(Window { internal: window });
        engine.world.create_resource(WindowSpecs {
            size: UVec2::new(window_size.0, window_size.1),
            ratio: window_size.0 as f32 / window_size.1 as f32,
            surface_id: 0,
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

    let mut runner_state = RunnerState::Initializing {
        app_ready: false,
        window_ready: false,
    };

    let event_handler = move |event: Event<()>,
                              _event_loop: &EventLoopWindowTarget<()>,
                              control_flow: &mut ControlFlow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::Resumed => {
                if runner_state == RunnerState::Suspended {
                    info!("Resume Engine");

                    if let Some(mut engine_event) =
                        engine.world.get_mut_event_handler::<EngineEvent>()
                    {
                        engine_event.publish(EngineEvent::Resumed);
                    }

                    let mut window_specs = engine.world.get_mut_resource::<WindowSpecs>().unwrap();
                    window_specs.surface_id += 1;

                    runner_state = RunnerState::Running;
                } else {
                    runner_state.set_app_ready();

                    if runner_state.can_start() {
                        runner_state = RunnerState::Running;
                        engine.startup();
                    }
                }
            }
            Event::Suspended => {
                info!("Supend Engine");

                runner_state = RunnerState::Suspending;
                if let Some(mut engine_event) = engine.world.get_mut_event_handler::<EngineEvent>()
                {
                    engine_event.publish(EngineEvent::Suspended);
                } else {
                    *control_flow = ControlFlow::Exit;
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
                if matches!(runner_state, RunnerState::Initializing { .. }) {
                    {
                        let mut window_specs =
                            engine.world.get_mut_resource::<WindowSpecs>().unwrap();
                        window_specs.size = UVec2::new(size.width, size.height);
                        window_specs.ratio = size.width as f32 / size.height as f32;
                    }

                    info!("New window size {:?}", size);

                    runner_state.set_window_ready();

                    if runner_state.can_start() {
                        runner_state = RunnerState::Running;
                        engine.startup();
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } if runner_state.is_running() => {
                if let Some(mut input) = engine.world.get_mut_event_handler::<InputEvent>() {
                    input.publish(InputEvent {
                        input: Input::MouseButton { button },
                        value: if state == ElementState::Pressed {
                            1.0
                        } else {
                            0.0
                        },
                    });
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } if runner_state.is_running() => {
                let window_specs = engine.world.get_resource::<WindowSpecs>().unwrap();
                if let Some(mut input) = engine.world.get_mut_event_handler::<InputEvent>() {
                    input.publish(InputEvent {
                        input: Input::MouseMotion { axis: Axis::X },
                        value: position.x as f32 / (window_specs.size.x / 2) as f32 - 1.,
                    });
                    input.publish(InputEvent {
                        input: Input::MouseMotion { axis: Axis::Y },
                        value: -1. * (position.y as f32 / (window_specs.size.y / 2) as f32 - 1.),
                    });
                }
            }
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } if runner_state.is_running() => match delta {
                MouseScrollDelta::LineDelta(x, y) => {
                    if let Some(mut input) = engine.world.get_mut_event_handler::<InputEvent>() {
                        input.publish(InputEvent {
                            input: Input::MouseWheel { axis: Axis::X },
                            value: x,
                        });
                        input.publish(InputEvent {
                            input: Input::MouseWheel { axis: Axis::Y },
                            value: y,
                        });
                    }
                }
                MouseScrollDelta::PixelDelta(p) => {
                    if let Some(mut input) = engine.world.get_mut_event_handler::<InputEvent>() {
                        input.publish(InputEvent {
                            input: Input::MouseWheel { axis: Axis::X },
                            value: p.x as f32,
                        });
                        input.publish(InputEvent {
                            input: Input::MouseWheel { axis: Axis::Y },
                            value: p.y as f32,
                        });
                    }
                }
            },
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input: keyboard_input,
                        ..
                    },
                ..
            } if runner_state.is_running() => {
                if let Some(mut input) = engine.world.get_mut_event_handler::<InputEvent>() {
                    input.publish(InputEvent {
                        input: Input::Keyboard {
                            key: keyboard_input.virtual_keycode.unwrap(),
                        },
                        value: if keyboard_input.state == ElementState::Pressed {
                            1.0
                        } else {
                            0.0
                        },
                    });
                }
            }
            Event::WindowEvent {
                event:
                    WindowEvent::Touch(winit::event::Touch {
                        phase, location, ..
                    }),
                ..
            } if runner_state.is_running() => {
                let window_specs = engine.world.get_resource::<WindowSpecs>().unwrap();
                if let Some(mut input) = engine.world.get_mut_event_handler::<InputEvent>() {
                    input.publish(InputEvent {
                        input: Input::Touch {
                            axis: Axis::X,
                            phase,
                        },
                        value: location.x as f32 / (window_specs.size.x / 2) as f32 - 1.,
                    });
                    input.publish(InputEvent {
                        input: Input::Touch {
                            axis: Axis::Y,
                            phase,
                        },
                        value: -1. * (location.y as f32 / (window_specs.size.y / 2) as f32 - 1.),
                    });
                }
            }
            Event::MainEventsCleared if runner_state.is_running() => {
                engine.update();

                if runner_state == RunnerState::Suspending {
                    runner_state = RunnerState::Suspended;
                }

                if engine
                    .world
                    .get_event_handler::<EngineEvent>()
                    .and_then(|event| event.read_last().map(|e| e == &EngineEvent::Quit))
                    .unwrap_or(false)
                {
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => (),
        }
    };

    event_loop.0.run(event_handler);
}

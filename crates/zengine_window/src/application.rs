use std::sync::{Arc, Mutex};

use glam::UVec2;
use log::info;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseScrollDelta, WindowEvent},
    event_loop::ActiveEventLoop,
    raw_window_handle::HasWindowHandle,
    window::{Fullscreen, WindowId},
};
use zengine_engine::{Engine, EngineEvent};
use zengine_input::{Axis, Input, InputEvent};

use crate::{RunnerState, Window, WindowConfig, WindowSpecs};

pub(crate) struct Application {
    engine: Engine,
    state: RunnerState,
}

impl Application {
    pub fn new(engine: Engine) -> Self {
        Self {
            engine,
            state: RunnerState::Initializing,
        }
    }

    fn create_window(&mut self, event_loop: &ActiveEventLoop) {
        let (window, window_spec) = {
            let window_config = self.engine.world.get_resource::<WindowConfig>().unwrap();

            let mut window_attributes = winit::window::Window::default_attributes()
                .with_title(window_config.title.clone())
                .with_inner_size(LogicalSize::new(window_config.width, window_config.height))
                .with_resizable(false);

            if window_config.fullscreen {
                window_attributes = window_attributes
                    .with_decorations(false)
                    .with_fullscreen(Some(Fullscreen::Borderless(None)));
            }

            let window = event_loop.create_window(window_attributes).unwrap();

            let window_size = if window_config.fullscreen {
                let size = window
                    .current_monitor()
                    .expect("No current monitor found")
                    .size();

                (size.width, size.height)
            } else {
                window.inner_size().into()
            };
            info!("size: {:?}", window_size);

            (
                window,
                WindowSpecs {
                    size: UVec2::new(window_size.0, window_size.1),
                    ratio: window_size.0 as f32 / window_size.1 as f32,
                    surface_id: 0,
                },
            )
        };

        self.engine.world.create_resource(window_spec);

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

        self.engine.world.create_unsendable_resource(Window {
            internal: Arc::new(window),
        });
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_is_none = self
            .engine
            .world
            .get_unsendable_resource::<Window>()
            .is_none();
        if window_is_none {
            self.create_window(event_loop);
        }

        if self.state == RunnerState::Suspended {
            info!("Resume Engine");

            if let Some(mut engine_event) = self.engine.world.get_mut_event_handler::<EngineEvent>()
            {
                engine_event.publish(EngineEvent::Resumed);
            }

            let mut window_specs = self.engine.world.get_mut_resource::<WindowSpecs>().unwrap();
            window_specs.surface_id += 1;

            self.state = RunnerState::Running;
        } else {
            self.state = RunnerState::Running;
            self.engine.startup();
        }
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        info!("Supend Engine");

        self.state = RunnerState::Suspending;
        if let Some(mut engine_event) = self.engine.world.get_mut_event_handler::<EngineEvent>() {
            engine_event.publish(EngineEvent::Suspended);
        } else {
            event_loop.exit();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                {
                    let mut window_specs =
                        self.engine.world.get_mut_resource::<WindowSpecs>().unwrap();
                    window_specs.size = UVec2::new(size.width, size.height);
                    window_specs.ratio = size.width as f32 / size.height as f32;
                    window_specs.surface_id += 1;
                }

                info!("New window size {:?}", size);
            }
            WindowEvent::MouseInput { state, button, .. } if self.state.is_running() => {
                if let Some(mut input) = self.engine.world.get_mut_event_handler::<InputEvent>() {
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
            WindowEvent::CursorMoved { position, .. } if self.state.is_running() => {
                let window_specs = self.engine.world.get_resource::<WindowSpecs>().unwrap();
                if let Some(mut input) = self.engine.world.get_mut_event_handler::<InputEvent>() {
                    input.publish(InputEvent {
                        input: Input::MouseMotion { axis: Axis::X },
                        value: position.x as f32 / (window_specs.size.x as f32 / 2.) - 1.,
                    });
                    input.publish(InputEvent {
                        input: Input::MouseMotion { axis: Axis::Y },
                        value: -1. * (position.y as f32 / (window_specs.size.y as f32 / 2.) - 1.),
                    });
                }
            }
            WindowEvent::MouseWheel { delta, .. } if self.state.is_running() => match delta {
                MouseScrollDelta::LineDelta(x, y) => {
                    if let Some(mut input) = self.engine.world.get_mut_event_handler::<InputEvent>()
                    {
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
                    if let Some(mut input) = self.engine.world.get_mut_event_handler::<InputEvent>()
                    {
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
            WindowEvent::KeyboardInput {
                event: keyboard_input,
                ..
            } if self.state.is_running() => {
                if let Some(mut input) = self.engine.world.get_mut_event_handler::<InputEvent>() {
                    if let winit::keyboard::PhysicalKey::Code(key) = keyboard_input.physical_key {
                        input.publish(InputEvent {
                            input: Input::Keyboard { key },
                            value: if keyboard_input.state == ElementState::Pressed {
                                1.0
                            } else {
                                0.0
                            },
                        });
                    }
                }
            }
            WindowEvent::Touch(winit::event::Touch {
                phase, location, ..
            }) if self.state.is_running() => {
                let window_specs = self.engine.world.get_resource::<WindowSpecs>().unwrap();
                if let Some(mut input) = self.engine.world.get_mut_event_handler::<InputEvent>() {
                    input.publish(InputEvent {
                        input: Input::Touch {
                            axis: Axis::X,
                            phase,
                        },
                        value: location.x as f32 / (window_specs.size.x as f32 / 2.) - 1.,
                    });
                    input.publish(InputEvent {
                        input: Input::Touch {
                            axis: Axis::Y,
                            phase,
                        },
                        value: -1. * (location.y as f32 / (window_specs.size.y as f32 / 2.) - 1.),
                    });
                }
            }
            WindowEvent::RedrawRequested => {
                self.engine.update();

                if self.state == RunnerState::Suspending {
                    self.state = RunnerState::Suspended;
                }

                if self
                    .engine
                    .world
                    .get_event_handler::<EngineEvent>()
                    .and_then(|event| event.read_last().map(|e| e == &EngineEvent::Quit))
                    .unwrap_or(false)
                {
                    event_loop.exit();
                } else if let Some(window) = self.engine.world.get_unsendable_resource::<Window>() {
                    window.internal.request_redraw();
                }
            }
            _ => (),
        }
    }
}

use std::sync::{Arc, Mutex};

use application::Application;
use glam::UVec2;
use log::info;
use winit::{
    dpi::LogicalSize, event::{ElementState, Event, MouseScrollDelta, WindowEvent}, event_loop::{ActiveEventLoop, ControlFlow}, raw_window_handle::RawWindowHandle, window::Fullscreen
};
use zengine_engine::{Engine, EngineEvent, Module};
use zengine_input::{Axis, Input, InputEvent};
use zengine_macro::{Resource, UnsendableResource};

#[cfg(target_os = "android")]
mod android_utils;

mod application;

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
    pub internal: Arc<winit::window::Window>,
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
    Initializing,
    Running,
    Suspending,
    Suspended,
}

impl RunnerState {
    pub fn is_running(&self) -> bool {
        matches!(self, RunnerState::Running | RunnerState::Suspending)
    }
}

///A [Module] that defines an interface for windowing support in ZENgine.
#[derive(Default, Debug)]
pub struct WindowModule(pub WindowConfig);
impl Module for WindowModule {
    fn init(self, engine: &mut Engine) {
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
        engine.set_runner(runner);
    }
}

fn runner(engine: Engine) {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = Application::new(engine);
    let _ = event_loop.run_app(&mut app);
}

extern crate gl;
extern crate sdl2;

use crate::basic::platform::resources::Platform;
use crate::core::system::ReadOption;
use crate::core::Store;
use crate::core::System;
use sdl2::video::{DisplayMode, FullscreenType, GLProfile, Window};
use sdl2::VideoSubsystem;

extern "system" fn dbg_callback(
    source: gl::types::GLenum,
    etype: gl::types::GLenum,
    _id: gl::types::GLuint,
    severity: gl::types::GLenum,
    _msg_length: gl::types::GLsizei,
    msg: *const gl::types::GLchar,
    _user_data: *mut std::ffi::c_void,
) {
    unsafe {
        println!(
            "dbg_callback {:#X} {:#X} {:#X} {:?}",
            source,
            etype,
            severity,
            std::ffi::CStr::from_ptr(msg),
        );
    }
}

//#[derive(Debug)]
pub struct WindowSystem {
    title: String,
    width: u32,
    height: u32,
    fullscreen: bool,
    window: Option<Window>,
}

impl WindowSystem {
    pub fn new(title: String, width: u32, height: u32, fullscreen: bool) -> Self {
        WindowSystem {
            title: title,
            width: width,
            height: height,
            fullscreen: fullscreen,
            window: None,
        }
    }
}

impl Default for WindowSystem {
    fn default() -> Self {
        WindowSystem {
            title: String::from("zengine"),
            width: 800,
            height: 600,
            fullscreen: false,
            window: None,
        }
    }
}

impl WindowSystem {
    fn get_display_mode(&self, video_subsystem: &VideoSubsystem) -> DisplayMode {
        for i in 0..video_subsystem.num_display_modes(0).unwrap() {
            let display_mode = video_subsystem.display_mode(0, i).unwrap();
            if display_mode.w == self.width as i32 && display_mode.h == self.height as i32 {
                return display_mode;
            }
        }

        panic!(
            "No DisplayMode available for width {} and height {}",
            self.width, self.height
        );
    }
}

impl<'a> System<'a> for WindowSystem {
    type Data = ReadOption<'a, Platform>;

    fn init(&mut self, store: &mut Store) {
        let platform = store
            .get_resource::<Platform>()
            .expect("No Platform resource found. Consider to register an EventPumpSystem");

        let video_subsystem = platform.context.video().unwrap();

        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        if cfg!(target_os = "macos") {
            gl_attr.set_context_version(4, 1);
        } else {
            gl_attr.set_context_version(4, 6);
        }
        gl_attr.set_double_buffer(true);

        let mut window = video_subsystem
            .window(self.title.as_ref(), self.width, self.height)
            .opengl()
            .allow_highdpi()
            .build()
            .unwrap();

        if self.fullscreen {
            let display_mode = self.get_display_mode(&video_subsystem);
            window.set_display_mode(display_mode).unwrap();
            window.set_fullscreen(FullscreenType::True).unwrap();
        }
        let _ctx = window.gl_create_context().unwrap();
        gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);
        unsafe {
            if !cfg!(target_os = "macos") {
                gl::Enable(gl::DEBUG_OUTPUT);
                gl::DebugMessageCallback(Some(dbg_callback), std::ptr::null());
            }
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
        println!(
            "Pixel format of the window's GL context: {:?}",
            window.window_pixel_format()
        );
        println!(
            "OpenGL Profile: {:?} - OpenGL version: {:?}",
            gl_attr.context_profile(),
            gl_attr.context_version()
        );

        self.window = Some(window);
    }

    fn run(&mut self, mut data: Self::Data) {}
}

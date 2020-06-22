extern crate log_panics;

use crate::core::scene::AnyScene;
use crate::core::system::AnySystem;
use crate::core::Store;
use crate::core::System;
use crate::core::{Scene, Trans};
use log::info;
use simplelog::{Config, LevelFilter, SimpleLogger, TermLogger, TerminalMode};

#[derive(Default)]
pub struct Engine {
    store: Store,
    systems: Vec<Box<dyn AnySystem>>,
}

impl Engine {
    pub fn init_logger(level_filter: LevelFilter) {
        if let Err(_) = TermLogger::init(level_filter, Config::default(), TerminalMode::Mixed) {
            SimpleLogger::init(level_filter, Config::default())
                .expect("An error occurred on logger initialization")
        }

        log_panics::init();
    }

    pub fn with_system<S: for<'a> System<'a>>(mut self, system: S) -> Self {
        self.systems.push(Box::new(system));

        self
    }

    pub fn run<S: AnyScene + 'static>(mut self, mut scene: S) {
        info!("Engine Start");

        info!("Init Systems");
        for s in self.systems.iter_mut() {
            s.init(&mut self.store);
        }
        info!("Scene Start");
        scene.on_start(&mut self.store);

        'main_loop: loop {
            for s in self.systems.iter_mut() {
                s.run(&self.store);
            }
            match scene.update(&self.store) {
                Trans::Quit => {
                    info!("Quit transaction received");
                    break 'main_loop;
                }
                _ => {}
            }
        }

        info!("Scene Stop");
        scene.on_stop(&mut self.store);

        info!("Dispose Systems");
        for s in self.systems.iter_mut() {
            s.dispose(&mut self.store);
        }

        info!("Engine Stop");
    }
}

/*
extern crate gl;
extern crate sdl2;

use crate::gl_utilities::shader::ShaderManager;
use crate::graphics::color::Color;
use crate::graphics::material::Material;
use crate::graphics::sprite::Sprite;
use crate::graphics::texture::Texture;
use crate::math::matrix4x4::Matrix4x4;
use crate::math::transform::Transform;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
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

pub struct EngineOption {
    pub title: String,
    pub fullscreen: bool,
    pub virtual_width: u32,
    pub virtual_height: u32,
    pub screen_width: u32,
    pub screen_height: u32,
}

pub fn start(option: EngineOption) {
    println!("Hello, ZENgine!");

    // Init Window
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    if cfg!(target_os = "macos") {
        gl_attr.set_context_version(4, 1);
    } else {
        gl_attr.set_context_version(4, 6);
    }
    gl_attr.set_double_buffer(true);

    let mut window = video_subsystem
        .window(
            option.title.as_ref(),
            option.screen_width,
            option.screen_height,
        )
        .opengl()
        .allow_highdpi()
        .build()
        .unwrap();

    if option.fullscreen {
        let display_mode = get_display_mode(&video_subsystem, &option);
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

    let projection = Matrix4x4::orthographics(
        0.0,
        option.virtual_width as f32,
        0.0,
        option.virtual_height as f32,
        -100.0,
        100.0,
    );

    let mut shader_manager = ShaderManager::init();
    let basic_shader = shader_manager.register(
        "basic",
        include_str!("basic.vert"),
        include_str!("basic.frag"),
    );

    let texture1 = Texture::new("test.png");
    let texture2 = Texture::new("duck.png");
    let u_projection_location = basic_shader.get_uniform_location("u_projection");

    let mut sprite = Sprite::new(
        "test",
        basic_shader,
        Material::new(Color::white(), &texture2),
        None,
        None,
    );
    sprite.load();

    let mut transform = Transform::new();
    transform.position.x = 150.0;
    transform.position.y = 500.0;

    transform.rotation.z = 0.0;
    transform.scale.x = 50.0;
    transform.scale.y = 50.0;

    basic_shader.use_shader();

    resize(&window, (option.virtual_width, option.virtual_height));

    let mut event_pump = sdl_context.event_pump().unwrap();

    'main_loop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'main_loop;
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    keymod,
                    ..
                } => match (keycode, keymod) {
                    (Keycode::R, _) => {
                        println!("red");
                        unsafe {
                            gl::ClearColor(1.0, 0.0, 0.0, 1.0);
                        }
                    }
                    (Keycode::G, _) => {
                        println!("green");
                        unsafe {
                            gl::ClearColor(0.0, 1.0, 0.0, 1.0);
                        }
                    }
                    (Keycode::B, _) => {
                        println!("blue");
                        unsafe {
                            gl::ClearColor(0.0, 0.0, 1.0, 1.0);
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
        }

        unsafe {
            gl::Disable(gl::SCISSOR_TEST);

            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::Enable(gl::SCISSOR_TEST);

            gl::ClearColor(1.0, 1.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::UniformMatrix4fv(
                u_projection_location,
                1,
                gl::FALSE,
                projection.data.as_ptr(),
            );

            sprite.draw(&transform.get_transformation_matrix());
        }
        window.gl_swap_window();
    }
}

fn resize(window: &Window, virtual_size: (u32, u32)) {
    let target_aspect_ratio = virtual_size.0 as f32 / virtual_size.1 as f32;

    let size = window.drawable_size();
    let width = size.0 as i32;
    let height = size.1 as i32;

    let mut calculated_height = (width as f32 / target_aspect_ratio) as i32;
    let mut calculated_width = width;

    if calculated_height > height {
        calculated_height = height;
        calculated_width = (calculated_height as f32 * target_aspect_ratio) as i32;
    }

    let vp_x = (width / 2) - (calculated_width / 2);
    let vp_y = (height / 2) - (calculated_height / 2);

    unsafe {
        gl::Viewport(vp_x, vp_y, calculated_width, calculated_height);
        gl::Scissor(vp_x, vp_y, calculated_width, calculated_height);
    }
}

fn get_display_mode(video_subsystem: &VideoSubsystem, option: &EngineOption) -> DisplayMode {
    for i in 0..video_subsystem.num_display_modes(0).unwrap() {
        let display_mode = video_subsystem.display_mode(0, i).unwrap();
        if display_mode.w == option.screen_width as i32
            && display_mode.h == option.screen_height as i32
        {
            return display_mode;
        }
    }

    panic!(
        "No DisplayMode available for width {} and height {}",
        option.screen_width, option.screen_height
    );
}
*/

extern crate sdl2;
extern crate gl;

mod gl_utility;
mod math;
mod graphics;

use sdl2::video::{GLProfile};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use gl_utility::shader::{ShaderManager};
use gl_utility::gl_buffer::{GLBuffer, AttributeInfo};
use math::matrix4x4::Matrix4x4;
use graphics::color::Color;

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

fn main() {
    println!("Hello, ZENgine!");

    let width = 800;
    let height = 600;

    // Init Window
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(4, 6);
    gl_attr.set_double_buffer(true);

    let window = video_subsystem
        .window(
            "ZENgine",
            width,
            height
        )
        .opengl()
        .build()
        .unwrap();

    let _ctx = window.gl_create_context().unwrap();
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

    unsafe {
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::DebugMessageCallback(Some(dbg_callback), std::ptr::null());
    }

    println!("Pixel format of the window's GL context: {:?}", window.window_pixel_format());
    println!("OpenGL Profile: {:?} - OpenGL version: {:?}", gl_attr.context_profile(), gl_attr.context_version());

    let projection = Matrix4x4::orthographics(0.0, width as f32, 0.0, height as f32, -100.0, 100.0);

    let mut shader_manager = ShaderManager::init();

    let basic_shader = shader_manager.register(
        "basic", 
        include_str!("basic.vert"), 
        include_str!("basic.frag")
    );

    let vertices: Vec<f32> = vec![
        //  x       y       z
            10.0,   10.0,   0.0, 
            10.0,   60.0,   0.0,  
            60.0,    60.0,    0.0,

            60.0,    60.0,    0.0,
            60.0,    10.0,   0.0,
            10.0,   10.0,   0.0
    ];
    
    let u_projection_location = basic_shader.get_uniform_location("u_projection");
    let a_position_location = basic_shader.get_attribute_location("a_position");
    let u_color_position = basic_shader.get_uniform_location("u_color");

    let mut buffer = GLBuffer::new();
    buffer.configure(
        vec![
            AttributeInfo {
                location: a_position_location,
                component_size: 3
            }
        ],
        false
    );

    buffer.set_data(vertices.as_slice());
    buffer.upload();

    basic_shader.use_shader();

    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    }
    window.gl_swap_window();

    let mut event_pump = sdl_context.event_pump().unwrap();
    
    let color = Color::new(255, 255, 0, 255);
    let color = Color::blue();

    'main_loop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'main_loop;
                },
                Event::KeyUp { keycode: Some(keycode), keymod, .. } => match(keycode, keymod) {
                    (Keycode::R, _) => {
                        println!("red");
                        unsafe {
                            gl::ClearColor(1.0, 0.0, 0.0, 1.0);
                        }
                    },
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
                    _ => ()
                }
                _ => ()
            }
        }

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // draw triangle
            gl::Uniform4f(
                u_color_position,
                color.r,
                color.g,
                color.b,
                color.a
            );

            gl::UniformMatrix4fv(
                u_projection_location,
                1,
                gl::FALSE,
                projection.data.as_ptr()
            );

            buffer.draw();
        }
        window.gl_swap_window();
    }
}

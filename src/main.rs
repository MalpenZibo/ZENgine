extern crate sdl2;
extern crate gl;

use sdl2::video::{GLProfile};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::ffi::{CString};

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

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    // allocate buffer of correct size
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    // fill it with len spaces
    buffer.extend([b' '].iter().cycle().take(len));
    // convert buffer to CString
    unsafe { CString::from_vec_unchecked(buffer) }
}

fn main() {
    println!("Hello, ZENgine!");

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
            800,
            600
        )
        .opengl()
        .build()
        .unwrap();

    let ctx = window.gl_create_context().unwrap();
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

    unsafe {
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::DebugMessageCallback(Some(dbg_callback), std::ptr::null());
    }

    println!("Pixel format of the window's GL context: {:?}", window.window_pixel_format());
    println!("OpenGL Profile: {:?} - OpenGL version: {:?}", gl_attr.context_profile(), gl_attr.context_version());

    // create vertex shader
    let vertex_id = unsafe { gl::CreateShader(gl::VERTEX_SHADER) };
    let vert_source = CString::new(include_str!("basic.vert")).expect("CString::new failed");

    unsafe {
        gl::ShaderSource(vertex_id, 1, &vert_source.as_ptr(), std::ptr::null());
        gl::CompileShader(vertex_id);
    }

    let mut success: gl::types::GLint = 1;
    unsafe {
        gl::GetShaderiv(vertex_id, gl::COMPILE_STATUS, &mut success);
    }

    if success == 0 {
        let mut len: gl::types::GLint = 0;
        unsafe {
            gl::GetShaderiv(vertex_id, gl::INFO_LOG_LENGTH, &mut len);
        }

        let error_msg = create_whitespace_cstring_with_len(len as usize);

        unsafe {
            gl::GetShaderInfoLog(
                vertex_id,
                len,
                std::ptr::null_mut(),
                error_msg.as_ptr() as *mut gl::types::GLchar
            );
        }

        println!("{}", error_msg.into_string().expect("into_string() failed"));
    }

    // create fragment shader
    let frag_id = unsafe { gl::CreateShader(gl::FRAGMENT_SHADER) };
    let frag_source = CString::new(include_str!("basic.frag")).expect("CString::new failed");

    unsafe {
        gl::ShaderSource(frag_id, 1, &frag_source.as_ptr(), std::ptr::null());
        gl::CompileShader(frag_id);
    }

    let mut success: gl::types::GLint = 1;
    unsafe {
        gl::GetShaderiv(frag_id, gl::COMPILE_STATUS, &mut success);
    }

    if success == 0 {
        let mut len: gl::types::GLint = 0;
        unsafe {
            gl::GetShaderiv(frag_id, gl::INFO_LOG_LENGTH, &mut len);
        }

        let error_msg = create_whitespace_cstring_with_len(len as usize);

        unsafe {
            gl::GetShaderInfoLog(
                frag_id,
                len,
                std::ptr::null_mut(),
                error_msg.as_ptr() as *mut gl::types::GLchar
            );
        }

        println!("{}", error_msg.into_string().expect("into_string() failed"));
    }

    // create program
    let program_id: gl::types::GLuint;
    unsafe {
        program_id = gl::CreateProgram();
        gl::AttachShader(program_id, vertex_id);
        gl::AttachShader(program_id, frag_id);

        gl::LinkProgram(program_id);
    }

    let mut success: gl::types::GLint = 1;
    unsafe {
        gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
    }

    if success == 0 {
        let mut len: gl::types::GLint = 0;
        unsafe {
            gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
        }

        let error_msg = create_whitespace_cstring_with_len(len as usize);

        unsafe {
            gl::GetProgramInfoLog(
                program_id,
                len,
                std::ptr::null_mut(),
                error_msg.as_ptr() as *mut gl::types::GLchar
            );
        }

        println!("{}", error_msg.into_string().expect("into_string() failed"));
    }

    unsafe {
        gl::DetachShader(program_id, vertex_id);
        gl::DetachShader(program_id, frag_id);
    }

    let vertices: Vec<f32> = vec![
        //  x       y       z
            -0.5,   -0.5,   0.0,
            -0.5,   0.5,   0.0,
            0.5,    0.5,    0.0,

            0.5,    0.5,    0.0,
            0.5,    -0.5,   0.0,
            -0.5,   -0.5,   0.0
    ];

    // setup vertex buffer object
    let mut vbo: gl::types::GLuint = 0;
    unsafe {
        gl::GenBuffers(1, &mut vbo);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, // size of data in bytes
            vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
            gl::STATIC_DRAW
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }

    // setup vertex array object
    let mut vao: gl::types::GLuint = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);

        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::EnableVertexAttribArray(0); // attribute a_position in basic.vert shader
        gl::VertexAttribPointer(
            0,  // index of vertex attribute (a_position)
            3,  // number of components per vertex attribute
            gl::FLOAT,   // data type
            gl::FALSE,  // normalized
            (3 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
            std::ptr::null()
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }

    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);

        // use program
        gl::UseProgram(program_id);
    }
    window.gl_swap_window();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut colors: Vec<f32> = vec![0.0, 0.0, 0.0, 1.0];
    let mut increment = 0.01;
    let mut index = 0;

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

            colors[index] += increment;

            if colors[index] >= 1.0 {
                colors[index] = 1.0;                

                if index == 2 {
                    increment *= -1.0;
                }
                else {
                    index += 1;
                }
            }
            if colors[index] < 0.0 {
                colors[index] = 0.0;
                
                if index == 0 {
                    increment *= -1.0;
                } 
                else {
                    index -= 1;
                }
            }

            gl::Uniform4fv(
                0,  // uniform position (u_color)
                1,
                colors.as_ptr() as *const gl::types::GLfloat
            );

            gl::BindVertexArray(vao);
            gl::DrawArrays(
                gl::TRIANGLES,   //mode
                0,              //starting index in the enabled arrays
                6               // number of indices
            );
        }
        window.gl_swap_window();
    }
}

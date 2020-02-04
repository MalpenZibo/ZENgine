use std::ffi::CString;
use std::collections::HashMap;

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    // allocate buffer of correct size
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    // fill it with len spaces
    buffer.extend([b' '].iter().cycle().take(len));
    // convert buffer to CString
    unsafe { CString::from_vec_unchecked(buffer) }
}

pub struct ShaderManager {
    shaders: HashMap<String, Shader>
}

impl ShaderManager {
    pub fn init() -> ShaderManager {
        ShaderManager {
            shaders: HashMap::new()
        }
    }

    pub fn register(&mut self, name: &str, vert_source: &str, frag_source: &str) -> &Shader {
        let mut shader = Shader {
            name: String::from(name),
            program: 0
        };

        shader.load(
            &CString::new(vert_source).expect("CString::new failed"),
            &CString::new(frag_source).expect("CString::new failed"),
        );

        self.shaders.insert(String::from(name), shader);

        self.get(name)
    }

    pub fn get(&self, name: &str) -> &Shader {
        match self.shaders.get(name) {
            Some(shader) => return shader,
            _ => panic!("Unable to find shader {}", name)
        };
    }
}

pub struct Shader {
    pub name: String,
    pub program: u32
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
        }
        println!("destroyed shader {}", self.name);
    }
}

impl Shader {
    pub fn use_shader(&self) {
        unsafe {
            gl::UseProgram(self.program);
        }
    }

    fn load(&mut self, vertex_source: &CString, fragment_source: &CString) {
        let vertex_shader = Shader::load_shader(vertex_source, gl::VERTEX_SHADER);
        let fragment_shader = Shader::load_shader(fragment_source, gl::FRAGMENT_SHADER);

        self.program = Shader::create_program(vertex_shader, fragment_shader);

        unsafe {
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
        }
    }

    fn load_shader(source: &CString, shader_type: gl::types::GLenum) -> u32 {
        let shader_id = unsafe { gl::CreateShader(shader_type) };

        unsafe {
            gl::ShaderSource(shader_id, 1, &source.as_ptr(), std::ptr::null());
            gl::CompileShader(shader_id);
        }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl::GetShaderiv(shader_id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error_msg = create_whitespace_cstring_with_len(len as usize);

            unsafe {
                gl::GetShaderInfoLog(
                    shader_id,
                    len,
                    std::ptr::null_mut(),
                    error_msg.as_ptr() as *mut gl::types::GLchar
                );
            }

            println!("{}", error_msg.into_string().expect("into_string() failed"));
        }

        shader_id
    }

    fn create_program(vertex_shader: u32, fragment_shader: u32) -> u32 {        
        let program_id = unsafe { gl::CreateProgram() };
        
        unsafe {            
            gl::AttachShader(program_id, vertex_shader);
            gl::AttachShader(program_id, fragment_shader);

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
            gl::DetachShader(program_id, vertex_shader);
            gl::DetachShader(program_id, fragment_shader);
        }

        program_id
    }
}
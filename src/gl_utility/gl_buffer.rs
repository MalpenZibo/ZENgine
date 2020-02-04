pub struct AttributeInfo {
    pub location: u32,
    pub component_size: i32
}

pub struct GLBuffer {
    type_size: usize,

    element_size: i32,
    stride: i32,

    vao: u32,
    vbo: u32,

    data: Vec<f32>
}

impl Drop for GLBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

impl GLBuffer {
    pub fn new() -> GLBuffer {
        let mut gl_buffer = GLBuffer {
            type_size: std::mem::size_of::<f32>(),

            element_size: 0,
            stride: 0,

            vao: 0,
            vbo: 0,

            data: Vec::new()
        };

        unsafe {
            gl::GenBuffers(1, &mut gl_buffer.vbo);
            gl::GenVertexArrays(1, &mut gl_buffer.vao);
        }

        gl_buffer
    }

    pub fn configure(&mut self, attributes: Vec<AttributeInfo>, normalized: bool) {
        unsafe {            
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            self.element_size = attributes.iter().map(|attribute| attribute.component_size).sum();
            self.stride = self.element_size * self.type_size as i32;

            let mut offset =  0;
            for attribute in &attributes {
                gl::VertexAttribPointer(
                    attribute.location,                     // index of vertex attribute 
                    attribute.component_size,               // number of components per vertex attribute
                    gl::FLOAT,                              // data type
                    normalized as gl::types::GLboolean,     // normalized
                    self.stride,                            // stride (byte offset between consecutive attributes)
                    (offset) as *const std::ffi::c_void     //offset in byte
                );
                gl::EnableVertexAttribArray(attribute.location); 
                
                offset += attribute.component_size;
            }

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }        
    }

    pub fn set_data(&mut self, data: &[f32]) {
        self.clear_data();
        self.push_back_data(data);
    }

    pub fn clear_data(&mut self) {
        self.data.clear();
    }

    pub fn push_back_data(&mut self, data: &[f32]) {
        for i in 0..data.len() {
            self.data.push(data[i]);
        }
    }

    pub fn upload(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.data.len() * self.type_size) as gl::types::GLsizeiptr, // size of data in bytes
                self.data.as_ptr() as *const gl::types::GLvoid,              // pointer to data
                gl::STATIC_DRAW
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }

    pub fn draw(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(
                gl::TRIANGLES,                              //mode
                0,                                          //starting index in the enabled arrays
                self.data.len() as i32 / self.element_size  // number of indices
            );
        }
    }
}
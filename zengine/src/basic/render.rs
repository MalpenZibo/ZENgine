use crate::core::system::Read;
use crate::core::system::ReadSet;
use crate::core::system::System;
use crate::core::Component;
use crate::core::Resource;
use crate::core::Store;
use crate::gl_utilities::gl_buffer::AttributeInfo;
use crate::gl_utilities::gl_buffer::GLBuffer;
use crate::gl_utilities::shader::Shader;
use crate::gl_utilities::shader::ShaderManager;
use crate::graphics::color::Color;
use crate::graphics::vertex::Vertex;
use crate::math::matrix4x4::Matrix4x4;
use crate::math::transform::Transform;
use crate::math::vector3::Vector3;
use log::info;
use sdl2::video::GLContext;
use sdl2::video::{DisplayMode, FullscreenType, GLProfile, Window};
use sdl2::VideoSubsystem;
use std::fmt::Debug;

pub struct WindowSpecs {
    title: String,
    width: u32,
    height: u32,
    fullscreen: bool,
}

impl WindowSpecs {
    pub fn new(title: String, width: u32, height: u32, fullscreen: bool) -> Self {
        WindowSpecs {
            title: title,
            width: width,
            height: height,
            fullscreen: fullscreen,
        }
    }
}

impl Default for WindowSpecs {
    fn default() -> Self {
        WindowSpecs {
            title: String::from("zengine"),
            width: 800,
            height: 600,
            fullscreen: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct Background {
    pub color: Color,
}
impl Resource for Background {}

#[derive(Debug)]
pub struct Sprite {
    pub width: f32,
    pub height: f32,
    pub origin: Vector3,
    pub color: Color,
}
impl Component for Sprite {}

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
        info!(
            "dbg_callback {:#X} {:#X} {:#X} {:?}",
            source,
            etype,
            severity,
            std::ffi::CStr::from_ptr(msg),
        );
    }
}

pub struct RenderSystem {
    window_specs: WindowSpecs,
    window: Option<Window>,
    ctx: Option<GLContext>,
    buffer: Option<GLBuffer>,
    //texture: Option<Texture>,
}

impl RenderSystem {
    pub fn new(specs: WindowSpecs) -> Self {
        RenderSystem {
            window_specs: specs,
            buffer: None,
            window: None,
            ctx: None,
            //texture: None,
        }
    }

    pub fn calculate_vertices(&mut self, width: f32, height: f32, origin: Vector3) {
        let min_x = -(width * origin.x);
        let max_x = width * (1.0 - origin.x);

        let min_y = -(height * origin.y);
        let max_y = height * (1.0 - origin.y);

        if let Some(buffer) = &mut self.buffer {
            buffer.upload(
                &[
                    Vertex::new(min_x, min_y, 0.0, 0.0, 0.0),
                    Vertex::new(min_x, max_y, 0.0, 0.0, 1.0),
                    Vertex::new(max_x, max_y, 0.0, 1.0, 1.0),
                    Vertex::new(max_x, max_y, 0.0, 1.0, 1.0),
                    Vertex::new(max_x, min_y, 0.0, 1.0, 0.0),
                    Vertex::new(min_x, min_y, 0.0, 0.0, 0.0),
                ]
                .iter()
                .flat_map(|v| {
                    vec![
                        v.position.x,
                        v.position.y,
                        v.position.z,
                        v.tex_coord.x,
                        v.tex_coord.y,
                    ]
                })
                .collect::<Vec<f32>>(),
            );
        }
    }

    fn render_sprites(
        &mut self,
        shader: &Shader,
        sprites: ReadSet<Sprite>,
        transforms: ReadSet<Transform>,
    ) {
        let u_color_position = shader.get_uniform_location("u_tint");
        let u_model_location = shader.get_uniform_location("u_model");
        //let u_diffuse_location = shader.get_uniform_location("u_diffuse");
        for s in sprites.iter() {
            if let Some(transform) = transforms.get(s.0) {
                self.calculate_vertices(s.1.width, s.1.height, s.1.origin);
                //if let Some(texture) = &self.texture {
                unsafe {
                    gl::UniformMatrix4fv(
                        u_model_location,
                        1,
                        gl::FALSE,
                        transform.get_transformation_matrix().data.as_ptr(),
                    );
                    gl::Uniform4f(
                        u_color_position,
                        s.1.color.r,
                        s.1.color.g,
                        s.1.color.b,
                        s.1.color.a,
                    );
                    //texture.activate();
                    //gl::Uniform1i(u_diffuse_location, 0);
                }
                if let Some(buffer) = &self.buffer {
                    buffer.draw();
                }
                //}
            }
        }
    }

    fn get_display_mode(&self, video_subsystem: &VideoSubsystem) -> DisplayMode {
        for i in 0..video_subsystem.num_display_modes(0).unwrap() {
            let display_mode = video_subsystem.display_mode(0, i).unwrap();
            if display_mode.w == self.window_specs.width as i32
                && display_mode.h == self.window_specs.height as i32
            {
                return display_mode;
            }
        }

        panic!(
            "No DisplayMode available for width {} and height {}",
            self.window_specs.width, self.window_specs.height
        );
    }
}

impl<'a> System<'a> for RenderSystem {
    type Data = (
        Read<'a, ShaderManager>,
        ReadSet<'a, Sprite>,
        ReadSet<'a, Transform>,
        Read<'a, Background>,
    );

    fn init(&mut self, store: &mut Store) {
        {
            let video_subsystem = store
                .get_resource::<VideoSubsystem>()
                .expect("No VideoSubsystem resource found. Consider to register an PlatformSystem");

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
                    self.window_specs.title.as_ref(),
                    self.window_specs.width,
                    self.window_specs.height,
                )
                .opengl()
                .allow_highdpi()
                .build()
                .unwrap();

            if self.window_specs.fullscreen {
                let display_mode = self.get_display_mode(&video_subsystem);
                window.set_display_mode(display_mode).unwrap();
                window.set_fullscreen(FullscreenType::True).unwrap();
            }

            self.ctx = Some(window.gl_create_context().unwrap());
            gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

            info!(
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

        unsafe {
            if !cfg!(target_os = "macos") {
                gl::Enable(gl::DEBUG_OUTPUT);
                gl::DebugMessageCallback(Some(dbg_callback), std::ptr::null());
            }
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        let mut shaders = ShaderManager::default();
        let basic_shader = shaders.register(
            "basic",
            include_str!("../basic.vert"),
            include_str!("../basic.frag"),
        );

        let a_position_location = basic_shader.get_attribute_location("a_position");
        let a_tex_coord_location = basic_shader.get_attribute_location("a_tex_coord");

        let mut buffer = GLBuffer::new();

        buffer.configure(
            vec![
                AttributeInfo {
                    location: a_position_location,
                    component_size: 3,
                },
                AttributeInfo {
                    location: a_tex_coord_location,
                    component_size: 2,
                },
            ],
            false,
        );

        //let texture = Texture::new("duck.png");
        //self.texture = Some(texture);

        self.buffer = Some(buffer);

        store.insert_resource(shaders);
    }

    fn run(&mut self, (shaders, sprites, transforms, background): Self::Data) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::ClearColor(
                background.color.r,
                background.color.g,
                background.color.b,
                background.color.a,
            );
        }
        let projection = Matrix4x4::orthographics(0.0, 800.0, 0.0, 600.0, -100.0, 100.0);

        let shader = shaders.get("basic");
        shader.use_shader();
        let u_projection_location = shader.get_uniform_location("u_projection");
        unsafe {
            gl::UniformMatrix4fv(
                u_projection_location,
                1,
                gl::FALSE,
                projection.data.as_ptr(),
            );
        }
        self.render_sprites(shader, sprites, transforms);

        if let Some(window) = &self.window {
            window.gl_swap_window();
        }
    }
}

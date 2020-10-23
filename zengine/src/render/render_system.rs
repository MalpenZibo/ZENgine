use crate::core::join::Join;
use crate::core::join::Optional;
use crate::core::system::ReadOption;
use crate::core::system::{Read, ReadSet, System};
use crate::core::Store;
use crate::gl_utilities::gl_buffer::AttributeInfo;
use crate::gl_utilities::gl_buffer::GLBuffer;
use crate::gl_utilities::shader::Shader;
use crate::gl_utilities::shader::ShaderManager;
use crate::graphics::camera::ActiveCamera;
use crate::graphics::camera::Camera;
use crate::graphics::texture::{SpriteType, TextureManager};
use crate::graphics::vertex::Vertex;
use crate::math::matrix4x4::Matrix4x4;
use crate::math::transform::Transform;
use crate::math::vector2::Vector2;
use crate::math::vector3::Vector3;
use crate::render::{Background, Sprite, WindowSpecs};
use log::info;
use sdl2::video::GLContext;
use sdl2::video::{DisplayMode, FullscreenType, GLProfile, Window};
use sdl2::VideoSubsystem;
use std::marker::PhantomData;

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

pub struct RenderSystem<ST: SpriteType> {
    window_specs: WindowSpecs,
    window: Option<Window>,
    ctx: Option<GLContext>,
    buffer: Option<GLBuffer>,
    sprite_type: PhantomData<ST>,
}

impl<ST: SpriteType> RenderSystem<ST> {
    pub fn new(specs: WindowSpecs) -> Self {
        RenderSystem {
            window_specs: specs,
            buffer: None,
            window: None,
            ctx: None,
            sprite_type: PhantomData,
        }
    }

    pub fn calculate_vertices(
        &mut self,
        width: f32,
        height: f32,
        origin: Vector3,
        relative_min: Vector2,
        relative_max: Vector2,
    ) {
        let min_x = -(width * origin.x);
        let max_x = width * (1.0 - origin.x);

        let min_y = -(height * origin.y);
        let max_y = height * (1.0 - origin.y);

        let min_u = relative_min.x;
        let max_u = relative_max.x;

        let min_v = relative_min.y;
        let max_v = relative_max.y;

        if let Some(buffer) = &mut self.buffer {
            buffer.upload(
                &[
                    Vertex::new(min_x, min_y, 0.0, min_u, max_v),
                    Vertex::new(min_x, max_y, 0.0, min_u, min_v),
                    Vertex::new(max_x, max_y, 0.0, max_u, min_v),
                    Vertex::new(max_x, max_y, 0.0, max_u, min_v),
                    Vertex::new(max_x, min_y, 0.0, max_u, max_v),
                    Vertex::new(min_x, min_y, 0.0, min_u, max_v),
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
        texture_manager: &TextureManager<ST>,
        shader: &Shader,
        sprites: ReadSet<Sprite<ST>>,
        transforms: ReadSet<Transform>,
    ) {
        let u_color_position = shader.get_uniform_location("u_tint");
        let u_model_location = shader.get_uniform_location("u_model");
        let u_diffuse_location = shader.get_uniform_location("u_diffuse");
        for s in sprites.iter() {
            if let Some(transform) = transforms.get(s.0) {
                let texture_handle = texture_manager.get_sprite_handle(&s.1.sprite_type).unwrap();
                self.calculate_vertices(
                    s.1.width,
                    s.1.height,
                    s.1.origin,
                    texture_handle.relative_min,
                    texture_handle.relative_max,
                );
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
                    texture_manager.activate(texture_handle.texture_id);
                    gl::Uniform1i(u_diffuse_location, 0);
                }
                if let Some(buffer) = &self.buffer {
                    buffer.draw();
                }
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

    fn create_window_and_opengl_context(&mut self, video_subsystem: &VideoSubsystem) {
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

    fn get_projection(
        &self,
        cameras: ReadSet<Camera>,
        active_camera: ReadOption<ActiveCamera>,
        transforms: &ReadSet<Transform>,
    ) -> Matrix4x4 {
        match active_camera {
            Some(active_camera) => match (
                cameras.get(&active_camera.entity),
                transforms.get(&active_camera.entity),
            ) {
                (Some(camera), Some(transform)) => {
                    camera.get_projection() * transform.get_transformation_matrix()
                }
                (Some(camera), None) => camera.get_projection(),
                _ => Matrix4x4::identity(),
            },
            None => match cameras.join(Optional(transforms)).next() {
                Some((_, camera, Some(transform))) => {
                    camera.get_projection() * transform.get_transformation_matrix()
                }
                Some((_, camera, None)) => camera.get_projection(),
                _ => Matrix4x4::identity(),
            },
        }
    }
}

type RenderData<'a, ST> = (
    Read<'a, TextureManager<ST>>,
    Read<'a, ShaderManager>,
    ReadSet<'a, Sprite<ST>>,
    ReadSet<'a, Transform>,
    Read<'a, Background>,
    ReadSet<'a, Camera>,
    ReadOption<'a, ActiveCamera>,
);

impl<'a, ST: SpriteType> System<'a> for RenderSystem<ST> {
    type Data = RenderData<'a, ST>;

    fn init(&mut self, store: &mut Store) {
        {
            let video_subsystem = store
                .get_resource::<VideoSubsystem>()
                .expect("No VideoSubsystem resource found. Consider to register an PlatformSystem");

            self.create_window_and_opengl_context(&video_subsystem);
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

        self.buffer = Some(buffer);

        store.insert_resource(TextureManager::<ST>::default());
        store.insert_resource(shaders);
    }

    fn run(
        &mut self,
        (texture_manager, shaders, sprites, transforms, background, camera, active_camera): Self::Data,
    ) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::ClearColor(
                background.color.r,
                background.color.g,
                background.color.b,
                background.color.a,
            );
        }
        let projection = self.get_projection(camera, active_camera, &transforms);

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
        self.render_sprites(&texture_manager, shader, sprites, transforms);

        if let Some(window) = &self.window {
            window.gl_swap_window();
        }
    }
}

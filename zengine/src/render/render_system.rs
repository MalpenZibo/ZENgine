use crate::core::join::Join;
use crate::core::system::ReadOption;
use crate::core::system::{Read, ReadSet, System};
use crate::core::Store;
use crate::gl_utilities::gl_buffer::AttributeInfo;
use crate::gl_utilities::gl_buffer::GlBuffer;
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
use crate::physics::collision::{Shape2D, ShapeType};
use crate::render::CollisionTrace;
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
    buffer: Option<GlBuffer>,
    sprite_type: PhantomData<ST>,
    collision_trace: CollisionTrace,
}

impl<ST: SpriteType> RenderSystem<ST> {
    pub fn new(specs: WindowSpecs, collision_trace: CollisionTrace) -> Self {
        RenderSystem {
            window_specs: specs,
            buffer: None,
            window: None,
            ctx: None,
            sprite_type: PhantomData,
            collision_trace,
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
        sprites: &ReadSet<Sprite<ST>>,
        transforms: &ReadSet<Transform>,
    ) {
        let u_color_position = shader.get_uniform_location("u_tint");
        let u_model_location = shader.get_uniform_location("u_model");
        let u_diffuse_location = shader.get_uniform_location("u_diffuse");
        for (_, sprite, transform) in sprites.join(transforms) {
            let texture_handle = texture_manager
                .get_sprite_handle(&sprite.sprite_type)
                .unwrap();
            self.calculate_vertices(
                sprite.width,
                sprite.height,
                sprite.origin,
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
                    sprite.color.r,
                    sprite.color.g,
                    sprite.color.b,
                    sprite.color.a,
                );
                texture_manager.activate(texture_handle.texture_id);
                gl::Uniform1i(u_diffuse_location, 0);
            }
            if let Some(buffer) = &self.buffer {
                buffer.draw();
            }
        }
    }

    fn render_shapes(
        &mut self,
        shader: &Shader,
        shapes: &ReadSet<Shape2D>,
        transforms: &ReadSet<Transform>,
    ) {
        let u_model_location = shader.get_uniform_location("u_model");
        let u_is_circle_location = shader.get_uniform_location("u_is_circle");

        for (_, shape, transform) in shapes.join(transforms) {
            let is_circle = match shape.shape_type {
                ShapeType::Circle { radius, .. } => {
                    self.calculate_vertices(
                        radius * 2.0,
                        radius * 2.0,
                        shape.origin,
                        Vector2::new(-1.0, -1.0),
                        Vector2::new(1.0, 1.0),
                    );

                    1
                }
                ShapeType::Rectangle { width, height, .. } => {
                    self.calculate_vertices(
                        width,
                        height,
                        shape.origin,
                        Vector2::new(0.0, 0.0),
                        Vector2::new(1.0, 1.0),
                    );

                    0
                }
            };

            unsafe {
                gl::Uniform1i(u_is_circle_location, is_circle);
                gl::UniformMatrix4fv(
                    u_model_location,
                    1,
                    gl::FALSE,
                    transform.get_transformation_matrix().data.as_ptr(),
                );
            }
            if let Some(buffer) = &self.buffer {
                buffer.draw();
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
            let display_mode = self.get_display_mode(video_subsystem);
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

    fn get_camera_data(
        &self,
        cameras: ReadSet<Camera>,
        active_camera: ReadOption<ActiveCamera>,
        transforms: &ReadSet<Transform>,
    ) -> (Matrix4x4, u32, u32) {
        match active_camera
            .map(|active| cameras.get_key_value(&active.entity))
            .flatten()
            .or_else(|| cameras.iter().next())
        {
            Some(camera) => (
                camera.1.get_projection()
                    * transforms
                        .get(camera.0)
                        .map(|transform| {
                            transform.get_transformation_matrix_inverse(true, true, false)
                        })
                        .unwrap_or_else(Matrix4x4::identity),
                camera.1.width,
                camera.1.height,
            ),
            None => (Matrix4x4::identity(), 1, 1),
        }
    }

    fn setup_scissor(&self, width: u32, height: u32) {
        if let Some(window) = &self.window {
            let target_aspect_ratio = width as f32 / height as f32;
            let size = window.drawable_size();
            let width = size.0 as i32;
            let height = size.1 as i32;
            let new_height = (width as f32 / target_aspect_ratio) as i32;
            let calculated_height = if new_height > height {
                height
            } else {
                new_height
            };
            let calculated_width = if new_height > height {
                (calculated_height as f32 * target_aspect_ratio) as i32
            } else {
                width
            };

            let vp_x = (width / 2) - (calculated_width / 2);
            let vp_y = (height / 2) - (calculated_height / 2);
            unsafe {
                gl::Viewport(vp_x, vp_y, calculated_width, calculated_height);
                gl::Scissor(vp_x, vp_y, calculated_width, calculated_height);
            }
        }
    }

    fn clear(&self, background: Read<Background>) {
        unsafe {
            gl::Disable(gl::SCISSOR_TEST);

            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::Enable(gl::SCISSOR_TEST);

            gl::ClearColor(
                background.color.r,
                background.color.g,
                background.color.b,
                background.color.a,
            );
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
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
    ReadSet<'a, Shape2D>,
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
            gl::Enable(gl::DEPTH_TEST);
        }

        let mut shaders = ShaderManager::default();
        let basic_shader = shaders.register(
            "basic",
            include_str!("../basic.vert"),
            include_str!("../basic.frag"),
        );
        let a_position_location = basic_shader.get_attribute_location("a_position");
        let a_tex_coord_location = basic_shader.get_attribute_location("a_tex_coord");

        let mut buffer = GlBuffer::new();

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

        shaders.register(
            "trace_shader",
            include_str!("../basic.vert"),
            include_str!("../trace.frag"),
        );

        store.insert_resource(TextureManager::<ST>::default());
        store.insert_resource(shaders);
    }

    fn run(
        &mut self,
        (texture_manager, shaders, sprites, transforms, background, camera, active_camera, shapes): Self::Data,
    ) {
        let camera_data = self.get_camera_data(camera, active_camera, &transforms);
        self.setup_scissor(camera_data.1, camera_data.2);
        self.clear(background);

        let shader = shaders.get("basic");
        shader.use_shader();
        let u_projection_location = shader.get_uniform_location("u_projection");
        unsafe {
            gl::UniformMatrix4fv(
                u_projection_location,
                1,
                gl::FALSE,
                camera_data.0.data.as_ptr(),
            );
        }
        self.render_sprites(&texture_manager, shader, &sprites, &transforms);

        if let CollisionTrace::Active = self.collision_trace {
            unsafe {
                gl::Disable(gl::DEPTH_TEST);
            }
            let shader = shaders.get("trace_shader");
            shader.use_shader();
            let u_projection_location = shader.get_uniform_location("u_projection");
            unsafe {
                gl::UniformMatrix4fv(
                    u_projection_location,
                    1,
                    gl::FALSE,
                    camera_data.0.data.as_ptr(),
                );
            }
            self.render_shapes(shader, &shapes, &transforms);
            unsafe {
                gl::Enable(gl::DEPTH_TEST);
            }
        }
        if let Some(window) = &self.window {
            window.gl_swap_window();
        }
    }
}

use crate::gl_utilities::gl_buffer::AttributeInfo;
use crate::gl_utilities::gl_buffer::GlBuffer;
use crate::gl_utilities::shader::Shader;
use crate::gl_utilities::shader::ShaderManager;
use crate::Background;
use crate::CollisionTrace;
use crate::Sprite;
use log::info;
use std::cell::Ref;
use std::cell::RefMut;
use std::fmt::Debug;
use std::marker::PhantomData;
use zengine_ecs::{
    system::{
        Commands, OptionalRes, OptionalUnsendableRes, OptionalUnsendableResMut, Query, QueryIter,
        Res,
    },
    Entity, UnsendableResource,
};
use zengine_graphic::{ActiveCamera, Camera, SpriteType, TextureManager, Vertex};
use zengine_math::{Matrix4x4, Transform, Vector2, Vector3};
use zengine_physics::{Shape2D, ShapeType};
use zengine_window::Window;

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

pub struct RenderContext<ST: SpriteType> {
    buffer: Option<GlBuffer>,
    sprite_type: PhantomData<ST>,
    collision_trace: CollisionTrace,
}

impl<ST: SpriteType> UnsendableResource for RenderContext<ST> {}

impl<ST: SpriteType> Debug for RenderContext<ST> {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

pub fn calculate_vertices<ST: SpriteType>(
    context: &mut RefMut<RenderContext<ST>>,
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

    if let Some(buffer) = &mut context.buffer {
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

fn render_sprites<ST: SpriteType>(
    context: &mut RefMut<RenderContext<ST>>,
    texture_manager: &TextureManager<ST>,
    shader: &Shader,
    sprite_query: Query<(&Transform, &Sprite<ST>)>,
) {
    let u_color_position = shader.get_uniform_location("u_tint");
    let u_model_location = shader.get_uniform_location("u_model");
    let u_diffuse_location = shader.get_uniform_location("u_diffuse");
    for (transform, sprite) in sprite_query.iter() {
        let texture_handle = texture_manager
            .get_sprite_handle(&sprite.sprite_type)
            .unwrap();
        calculate_vertices(
            context,
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
        if let Some(buffer) = &context.buffer {
            buffer.draw();
        }
    }
}

fn render_shapes<ST: SpriteType>(
    context: &mut RefMut<RenderContext<ST>>,
    shader: &Shader,
    shape_query: Query<(&Transform, &Shape2D)>,
) {
    let u_model_location = shader.get_uniform_location("u_model");
    let u_is_circle_location = shader.get_uniform_location("u_is_circle");

    for (transform, shape) in shape_query.iter() {
        let is_circle = match shape.shape_type {
            ShapeType::Circle { radius, .. } => {
                calculate_vertices(
                    context,
                    radius * 2.0,
                    radius * 2.0,
                    shape.origin,
                    Vector2::new(-1.0, -1.0),
                    Vector2::new(1.0, 1.0),
                );

                1
            }
            ShapeType::Rectangle { width, height, .. } => {
                calculate_vertices(
                    context,
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
        if let Some(buffer) = &context.buffer {
            buffer.draw();
        }
    }
}

fn create_window_and_opengl_context(window: &Window) {
    gl::load_with(|symbol| window.get_proc_address(symbol));

    info!(
        "Pixel format of the window's GL context: {:?}",
        window.get_pixel_format()
    );
    println!("OpenGL {:?}", window.context().get_api());
}

fn get_camera_data(
    camera_query: Query<(Entity, &Transform, &Camera)>,
    active_camera: OptionalRes<ActiveCamera>,
) -> (Matrix4x4, u32, u32) {
    match active_camera
        .and_then(|active| {
            camera_query.iter().find_map(|(e, t, c)| {
                if e == &active.entity {
                    Some((e, t, c))
                } else {
                    None
                }
            })
        })
        .or_else(|| camera_query.iter().next())
    {
        Some((_, transform, camera)) => (
            camera.get_projection()
                * transform.get_transformation_matrix_inverse(true, true, false),
            camera.width,
            camera.height,
        ),
        None => (Matrix4x4::identity(), 1, 1),
    }
}

fn setup_scissor(window: &Option<Ref<Window>>, width: u32, height: u32) {
    if let Some(window) = &window {
        let target_aspect_ratio = width as f32 / height as f32;
        let size = window.window().inner_size();
        let width = size.width as i32;
        let height = size.height as i32;
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

fn clear(background: Res<Background>) {
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

pub fn setup_render<ST: SpriteType>(
    collision_trace: CollisionTrace,
) -> impl Fn(OptionalUnsendableRes<Window>, Commands) {
    move |window: OptionalUnsendableRes<Window>, mut command: Commands| {
        let mut context = RenderContext::<ST> {
            buffer: None,
            sprite_type: PhantomData::default(),
            collision_trace,
        };
        create_window_and_opengl_context(&window.expect("window not present"));

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
            include_str!("./basic.vert"),
            include_str!("./basic.frag"),
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

        context.buffer = Some(buffer);

        shaders.register(
            "trace_shader",
            include_str!("./basic.vert"),
            include_str!("./trace.frag"),
        );

        command.create_resource(TextureManager::<ST>::default());
        command.create_resource(shaders);
        command.create_unsendable_resource(context);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn render_system<ST: SpriteType>(
    window: OptionalUnsendableRes<Window>,
    context: OptionalUnsendableResMut<RenderContext<ST>>,
    texture_manager: Res<TextureManager<ST>>,
    shaders: Res<ShaderManager>,
    background: Res<Background>,
    active_camera: OptionalRes<ActiveCamera>,
    sprite_query: Query<(&Transform, &Sprite<ST>)>,
    camera_query: Query<(Entity, &Transform, &Camera)>,
    shape_query: Query<(&Transform, &Shape2D)>,
) {
    let mut context = context.unwrap();

    let camera_data = get_camera_data(camera_query, active_camera);
    setup_scissor(&window, camera_data.1, camera_data.2);
    clear(background);

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
    render_sprites(&mut context, &texture_manager, shader, sprite_query);

    if let CollisionTrace::Active = context.collision_trace {
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
        render_shapes(&mut context, shader, shape_query);
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
        }
    }
    if let Some(current_window) = &window {
        current_window.swap_buffers().unwrap();
    }
}

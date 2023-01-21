use crate::{Device, Queue};
use glam::Vec2;
use std::ops::MulAssign;
use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout};
use zengine_core::Transform;
use zengine_ecs::{
    query::{Query, QueryIter},
    system::{Commands, Res, ResMut},
    Entity,
};
use zengine_macro::{Component, Resource};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        }
    }
}

impl CameraUniform {
    pub fn new(camera: &Camera, transform: Option<&Transform>) -> CameraUniform {
        Self {
            view_proj: camera.get_projection(transform).to_cols_array_2d(),
        }
    }
}

/// [Resource](zengine_ecs::Resource) that defines the current used camera
#[derive(Resource, Debug)]
pub struct ActiveCamera {
    /// Entity that have the active component camera attached
    pub entity: Entity,
}

/// Type of camera
#[derive(Debug)]
pub enum CameraMode {
    /// Defines an orthographic 2D projection using
    /// the given Vec2 to set the viewbox width and height
    Mode2D(Vec2),
}

/// [Component](zengine_ecs::Component) that rappresent a Camera
///
/// Contains a [CameraMode] that define the type of projection
/// (currently only a 2D camera could be defined)
///
/// # Example
/// ```
/// use zengine_ecs::system::Commands;
/// use zengine_graphic::{Camera, CameraMode};
/// use glam::Vec2;
///
/// fn setup(mut commands: Commands) {
///     commands.spawn(
///         Camera {
///             mode: CameraMode::Mode2D(Vec2::new(3.55, 2.0))
///         }
///     );
/// }
/// ```
#[derive(Component, Debug)]
pub struct Camera {
    pub mode: CameraMode,
}

impl Camera {
    /// Get the [Camera] size
    pub fn get_size(&self) -> Vec2 {
        match self.mode {
            CameraMode::Mode2D(size) => size,
        }
    }

    pub(crate) fn get_projection(&self, transform: Option<&Transform>) -> glam::Mat4 {
        let mut proj = match self.mode {
            CameraMode::Mode2D(size) => glam::Mat4::orthographic_lh(
                -size.x / 2.0,
                size.x / 2.0,
                -size.y / 2.0,
                size.y / 2.0,
                0.0,
                1000.0,
            ),
        };

        if let Some(transform) = transform {
            proj.mul_assign(transform.get_transformation_matrix().inverse());
        }

        proj
    }
}

#[derive(Resource, Debug)]
pub struct CameraBuffer {
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

#[derive(Debug)]
struct UsedCameraData {
    pub size: Vec2,
    pub transform: Option<Transform>,
}

/// Resource that contains information about the Camera used to render the current frame
#[derive(Resource, Default, Debug)]
pub struct UsedCamera(Option<UsedCameraData>);

impl UsedCamera {
    /// Get the size of the [Camera] used to render the current frame
    pub fn get_size(&self) -> Option<Vec2> {
        self.0.as_ref().map(|d| d.size)
    }

    /// Get the transform of the [Camera] used to render the current frame
    pub fn get_transform(&self) -> Option<&Transform> {
        self.0.as_ref().and_then(|d| d.transform.as_ref())
    }
}

fn pick_correct_camera<'a>(
    camera_query: &'a Query<(Entity, &Camera, Option<&Transform>)>,
    active_camera: &'a Option<Res<ActiveCamera>>,
) -> Option<(&'a Camera, Option<&'a Transform>)> {
    active_camera
        .as_ref()
        .map_or_else(
            || camera_query.iter().next(),
            |active| camera_query.iter().find(|(e, ..)| **e == active.entity),
        )
        .map(|(_, c, t)| (c, t))
}

pub(crate) fn setup_camera(device: Option<Res<Device>>, mut commands: Commands) {
    let device = device.unwrap();

    let camera_uniform = CameraUniform::default();

    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Camera Buffer"),
        contents: bytemuck::cast_slice(&[camera_uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some("camera_bind_group_layout"),
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
        label: Some("camera_bind_group"),
    });

    commands.create_resource(CameraBuffer {
        buffer,
        bind_group_layout,
        bind_group,
    })
}

pub(crate) fn camera_render(
    queue: Option<Res<Queue>>,
    camera_query: Query<(Entity, &Camera, Option<&Transform>)>,
    active_camera: Option<Res<ActiveCamera>>,
    camera_buffer: Option<ResMut<CameraBuffer>>,
    mut used_camera: ResMut<UsedCamera>,
) {
    if let (Some(queue), Some(camera_buffer)) = (queue, camera_buffer) {
        let camera_data = pick_correct_camera(&camera_query, &active_camera);

        used_camera.0 = camera_data.map(|(c, t)| UsedCameraData {
            size: c.get_size(),
            transform: t.map(|t| t.clone()),
        });

        let camera_uniform = if let Some((camera, transform)) = camera_data {
            CameraUniform::new(camera, transform)
        } else {
            CameraUniform::default()
        };

        queue.write_buffer(
            &camera_buffer.buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        )
    }
}

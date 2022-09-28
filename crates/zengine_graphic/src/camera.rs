use std::ops::MulAssign;

use glam::Vec2;
use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout};
use zengine_core::Transform;
use zengine_ecs::{
    system::{Commands, OptionalRes, Query, QueryIter},
    Entity,
};
use zengine_macro::{Component, Resource};

use crate::{Device, Queue};

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

#[derive(Resource, Debug)]
pub struct ActiveCamera {
    pub entity: Entity,
}

#[derive(Debug)]
pub enum CameraMode {
    Mode2D(Vec2),
}

#[derive(Component, Debug)]
pub struct Camera {
    pub mode: CameraMode,
}

impl Camera {
    pub fn get_projection(&self, transform: Option<&Transform>) -> glam::Mat4 {
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

fn pick_correct_camera<'a>(
    camera_query: &'a Query<(Entity, &Camera, Option<&Transform>)>,
    active_camera: &'a OptionalRes<ActiveCamera>,
) -> Option<(&'a Camera, Option<&'a Transform>)> {
    active_camera
        .as_ref()
        .map_or_else(
            || camera_query.iter().next(),
            |active| camera_query.iter().find(|(e, ..)| **e == active.entity),
        )
        .map(|(_, c, t)| (c, t))
}

pub fn setup_camera(device: OptionalRes<Device>, mut commands: Commands) {
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

pub fn camera_render(
    queue: OptionalRes<Queue>,
    camera_query: Query<(Entity, &Camera, Option<&Transform>)>,
    active_camera: OptionalRes<ActiveCamera>,
    camera_buffer: OptionalRes<CameraBuffer>,
) {
    if let (Some(queue), Some(camera_buffer)) = (queue, camera_buffer) {
        let camera_data = pick_correct_camera(&camera_query, &active_camera);
        if let Some((camera, transform)) = camera_data {
            queue.write_buffer(
                &camera_buffer.buffer,
                0,
                bytemuck::cast_slice(&[CameraUniform::new(camera, transform)]),
            );
        } else {
            queue.write_buffer(
                &camera_buffer.buffer,
                0,
                bytemuck::cast_slice(&[CameraUniform::default()]),
            );
        }
    }
}

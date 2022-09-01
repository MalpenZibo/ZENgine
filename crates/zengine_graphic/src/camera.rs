use zengine_core::Transform;
use zengine_ecs::Entity;
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

#[rustfmt::skip]
// pub const OPENGL_TO_WGPU_MATRIX: glm::Mat4 = glm::Mat4::new(
//     1.0, 0.0, 0.0, 0.0,
//     0.0, 1.0, 0.0, 0.0,
//     0.0, 0.0, 0.5, 0.0,
//     0.0, 0.0, 0.5, 1.0,
// );

#[derive(Resource, Debug)]
pub struct ActiveCamera {
    pub entity: Entity,
}

type CameraSize = (f32, f32);

#[derive(Debug)]
pub enum CameraMode {
    Mode2D(CameraSize),
}

#[derive(Component, Debug)]
pub struct Camera {
    pub mode: CameraMode,
}

impl Camera {
    pub fn get_projection(&self, transform: Option<&Transform>) -> glam::Mat4 {
        let proj = match self.mode {
            CameraMode::Mode2D((width, height)) => glam::Mat4::orthographic_lh(
                -width / 2.0,
                width / 2.0,
                -height / 2.0,
                height / 2.0,
                0.0,
                1000.0,
            ),
        };

        if let Some(transform) = transform {
            proj * transform.get_transformation_matrix().inverse() // * glam::Mat4::look_at_lh(*position, glam::Vec3::ZERO, glam::Vec3::Y)
        } else {
            proj
        }

        // match &self.mode {
        //     CameraMode::Mode2D => Matrix4x4::orthographics(
        //         -(self.width as f32 / 2.0),
        //         self.width as f32 / 2.0,
        //         -(self.height as f32 / 2.0),
        //         self.height as f32 / 2.0,
        //         0.0,
        //         1000.0,
        //     ),
        // }
    }
}

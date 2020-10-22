use crate::core::Component;
use crate::core::Entity;
use crate::core::Resource;
use crate::math::matrix4x4::Matrix4x4;

#[derive(Resource, Debug)]
pub struct ActiveCamera {
    pub entity: Entity,
}

#[derive(Debug)]
pub enum CameraMode {
    MODE_2D,
}

#[derive(Component, Debug)]
pub struct Camera {
    pub width: u32,
    pub height: u32,
    pub mode: CameraMode,
}

impl Camera {
    pub fn get_projection(&self) -> Matrix4x4 {
        match &self.mode {
            MODE_2D => Matrix4x4::orthographics(
                0.0,
                self.width as f32,
                0.0,
                self.height as f32,
                -100.0,
                100.0,
            ),
        }
    }
}

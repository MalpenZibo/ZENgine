use zengine_ecs::Entity;
use zengine_macro::{Component, Resource};
use zengine_math::Matrix4x4;

#[derive(Resource, Debug)]
pub struct ActiveCamera {
    pub entity: Entity,
}

#[derive(Debug)]
pub enum CameraMode {
    Mode2D,
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
            CameraMode::Mode2D => Matrix4x4::orthographics(
                -(self.width as f32 / 2.0),
                self.width as f32 / 2.0,
                -(self.height as f32 / 2.0),
                self.height as f32 / 2.0,
                0.0,
                1000.0,
            ),
        }
    }
}
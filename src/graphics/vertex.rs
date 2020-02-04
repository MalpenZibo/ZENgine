use crate::math::vector3::Vector3;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: Vector3
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32) -> Vertex {
        Vertex {
            position: Vector3::new(x, y, z)
        }
    }
}
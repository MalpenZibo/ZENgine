use crate::math::vector2::Vector2;
use crate::math::vector3::Vector3;

#[derive(Default, Copy, Clone)]
pub struct Vertex {
    pub position: Vector3,
    pub tex_coord: Vector2,
}

impl Vertex {
    pub fn new(pos_x: f32, pos_y: f32, pos_z: f32, tex_u: f32, tex_v: f32) -> Vertex {
        Vertex {
            position: Vector3::new(pos_x, pos_y, pos_z),
            tex_coord: Vector2::new(tex_u, tex_v),
        }
    }
}

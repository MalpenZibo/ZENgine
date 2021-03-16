use auto_ops::*;

impl_op_ex!(-|a: &Vector3, b: &Vector3| -> Vector3 {
    Vector3::new(a.x - b.x, a.y - b.y, a.z - b.z)
});

impl_op_ex!(-=|a: &mut Vector3, b: &Vector3| {
    a.x - b.x;
    a.y - b.y;
    a.z - b.z;
});

impl_op_ex!(+|a: &Vector3, b: &Vector3| -> Vector3 {
    Vector3::new(a.x + b.x, a.y + b.y, a.z + b.z)
});

impl_op_ex!(+=|a: &mut Vector3, b: &Vector3| {
    a.x + b.x;
    a.y + b.y;
    a.z + b.z;
});

#[derive(Copy, Clone, Debug, Default)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vector3 {
        Vector3 { x, y, z }
    }

    pub fn zero() -> Vector3 {
        Vector3::new(0.0, 0.0, 0.0)
    }

    pub fn one() -> Vector3 {
        Vector3::new(1.0, 1.0, 1.0)
    }

    pub fn inverse(&self) -> Vector3 {
        Vector3::new(-self.x, -self.y, -self.z)
    }

    pub fn distance(&self, other: &Self) -> f32 {
        let diff = self - other;
        return f32::sqrt(diff.x * diff.x + diff.y * diff.y + diff.z * diff.z);
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Vector2 {
        Vector2 { x, y }
    }

    pub fn zero() -> Vector2 {
        Vector2::new(0.0, 0.0)
    }

    pub fn one() -> Vector2 {
        Vector2::new(1.0, 1.0)
    }

    pub fn inverse(&self) -> Vector2 {
        Vector2::new(-self.x, -self.y)
    }
}

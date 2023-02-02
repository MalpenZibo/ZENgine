/// Describe an RGB color
#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Color::WHITE
    }
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: Self::to_linear_rgb(r),
            g: Self::to_linear_rgb(g),
            b: Self::to_linear_rgb(b),
            a: a as f32 / 255.0,
        }
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub const WHITE: Self = Self {
        r: 1.,
        g: 1.,
        b: 1.,
        a: 1.,
    };

    pub const BLACK: Self = Self {
        r: 0.,
        g: 0.,
        b: 0.,
        a: 1.,
    };

    pub const RED: Self = Self {
        r: 1.,
        g: 0.,
        b: 0.,
        a: 1.,
    };

    pub const GREEN: Self = Self {
        r: 0.,
        g: 1.,
        b: 0.,
        a: 1.,
    };

    pub const BLUE: Self = Self {
        r: 0.,
        g: 0.,
        b: 1.,
        a: 1.,
    };

    fn to_linear_rgb(xu: u8) -> f32 {
        let x = xu as f32 / 255.0;
        if x > 0.04045 {
            ((x + 0.055) / 1.055).powf(2.4)
        } else {
            x / 12.92
        }
    }
}

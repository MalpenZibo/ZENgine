#[derive(Debug)]
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
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub const WHITE: Self = Color {
        r: 1.,
        g: 1.,
        b: 1.,
        a: 1.,
    };

    pub const BLACK: Self = Color {
        r: 0.,
        g: 0.,
        b: 0.,
        a: 1.,
    };

    pub const RED: Self = Color {
        r: 1.,
        g: 0.,
        b: 0.,
        a: 1.,
    };

    pub const GREEN: Self = Color {
        r: 0.,
        g: 1.,
        b: 0.,
        a: 1.,
    };

    pub const BLUE: Self = Color {
        r: 0.,
        g: 0.,
        b: 1.,
        a: 1.,
    };
}

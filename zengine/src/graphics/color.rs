pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32
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

    pub fn white() -> Color {
        Color::new(255, 255, 255, 255)
    }

    pub fn black() -> Color {
        Color::new(0, 0, 0, 255)
    }

    pub fn red() -> Color {
        Color::new(255, 0, 0, 255)
    }

    pub fn green() -> Color {
        Color::new(0, 255, 0, 255)
    }

    pub fn blue() -> Color {
        Color::new(0, 0, 255, 255)
    }
}
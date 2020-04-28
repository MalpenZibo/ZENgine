use crate::graphics::texture::Texture;
use crate::graphics::color::Color;

pub struct Material<'a> {
    pub tint: Color,
    pub texture: &'a Texture
}

impl<'a> Material<'a> {
    pub fn new(tint: Color, texture: &'a Texture) -> Material {
        Material {
            tint: tint,
            texture: texture
        }
    }
}
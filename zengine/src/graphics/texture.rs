use crate::assets::image_loader;
use crate::core::Resource;
use crate::math::vector2::Vector2;
use fnv::FnvHashMap;
use std::any::Any;
use std::fmt::Debug;
use std::hash::Hash;

const LEVEL: i32 = 0;
const BORDER: i32 = 0;

pub trait SpriteType: Any + Eq + PartialEq + Hash + Clone + Debug {}
impl SpriteType for String {}

pub struct SpriteDescriptor {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub struct SpriteHandle {
    pub relative_min: Vector2,
    pub relative_max: Vector2,
    pub width: u32,
    pub height: u32,
    pub texture_id: u32,
}

pub struct TextureLoader<'a, ST: SpriteType> {
    file_path: String,
    texture_manager: &'a mut TextureManager<ST>,
    sprites: FnvHashMap<ST, SpriteDescriptor>,
}

impl<'a, ST: SpriteType> TextureLoader<'a, ST> {
    pub fn with_sprite(mut self, sprite_type: ST, descriptor: SpriteDescriptor) -> Self {
        self.sprites.insert(sprite_type, descriptor);

        self
    }

    pub fn load(self) {
        self.texture_manager.load(&self.file_path, self.sprites)
    }
}

#[derive(Resource)]
pub struct TextureManager<ST: SpriteType> {
    sprites: FnvHashMap<ST, SpriteHandle>,
}

impl<ST: SpriteType> Default for TextureManager<ST> {
    fn default() -> Self {
        TextureManager {
            sprites: FnvHashMap::default(),
        }
    }
}

impl<ST: SpriteType> TextureManager<ST> {
    pub(self) fn load(&mut self, file_path: &str, sprites: FnvHashMap<ST, SpriteDescriptor>) {
        let img = image_loader::load(file_path);
        let texture_id = Self::generate_texture(img.width, img.height, &img.data);

        for entry in sprites {
            if let Some(old_sprite) = self.sprites.insert(
                entry.0,
                SpriteHandle {
                    relative_min: Vector2::new(
                        entry.1.x as f32 / img.width as f32,
                        entry.1.y as f32 / img.height as f32,
                    ),
                    relative_max: Vector2::new(
                        (entry.1.x + entry.1.width) as f32 / img.width as f32,
                        (entry.1.y + entry.1.height) as f32 / img.height as f32,
                    ),
                    width: entry.1.width,
                    height: entry.1.height,
                    texture_id,
                },
            ) {
                if !self.texture_still_used(old_sprite.texture_id) {
                    Self::destroy_texture(old_sprite.texture_id);
                }
            }
        }
    }

    pub fn create(&mut self, file_path: &str) -> TextureLoader<ST> {
        TextureLoader {
            texture_manager: self,
            file_path: file_path.to_string(),
            sprites: FnvHashMap::default(),
        }
    }

    pub fn destroy(&mut self, sprite_type: ST) {
        if let Some(sprite) = self.sprites.remove(&sprite_type) {
            if self.texture_still_used(sprite.texture_id) {
                Self::destroy_texture(sprite.texture_id);
            }
        }
    }

    pub fn get_sprite_handle(&self, sprite_type: &ST) -> Option<&SpriteHandle> {
        self.sprites.get(sprite_type)
    }

    pub fn activate(&self, texture_id: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
        }
    }

    fn texture_still_used(&self, texture_id: u32) -> bool {
        self.sprites
            .values()
            .any(|sprite| sprite.texture_id == texture_id)
    }

    fn destroy_texture(texture_id: u32) {
        unsafe {
            gl::DeleteTextures(1, &texture_id);
        }
    }

    fn generate_texture(width: u32, height: u32, data: &Vec<u8>) -> u32 {
        let mut texture_id = 0;
        unsafe {
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                LEVEL,
                gl::RGBA as i32,
                width as i32,
                height as i32,
                BORDER,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const gl::types::GLvoid,
            );

            gl::GenerateMipmap(gl::TEXTURE_2D);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        texture_id
    }
}

use crate::assets::image_loader;
use crate::core::Resource;
use crate::math::vector2::Vector2;
use fnv::FnvHashMap;
use std::any::Any;
use std::fmt::Debug;
use std::hash::Hash;

const LEVEL: i32 = 0;
const BORDER: i32 = 0;

pub trait TextureType: Any + Eq + PartialEq + Hash + Clone + Debug {
    fn get_path(&self) -> &str;
}
impl TextureType for String {
    fn get_path(&self) -> &str {
        self
    }
}

pub trait SpriteType: Any + Eq + PartialEq + Hash + Clone + Debug {}
impl SpriteType for String {}

pub type SpriteSheet<T> = FnvHashMap<T, SpriteDescriptor>;

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

pub struct TextureLoader<'a, TT: TextureType, ST: SpriteType> {
    texture_type: TT,
    texture_manager: &'a mut TextureManager<TT, ST>,
    sprites: FnvHashMap<ST, SpriteDescriptor>,
}

impl<'a, TT: TextureType, ST: SpriteType> TextureLoader<'a, TT, ST> {
    pub fn with_sprite(mut self, sprite_type: ST, descriptor: SpriteDescriptor) -> Self {
        self.sprites.insert(sprite_type, descriptor);

        self
    }

    pub fn load(self) -> Result<(), TextureError> {
        self.texture_manager.load(self.texture_type, self.sprites)
    }
}

#[derive(Resource)]
pub struct TextureManager<TT: TextureType, ST: SpriteType> {
    textures: FnvHashMap<TT, u32>,
    sprites: FnvHashMap<ST, SpriteHandle>,
}

pub enum TextureError {
    AlreadyLoaded,
    NotLoaded,
}

impl<TT: TextureType, ST: SpriteType> Default for TextureManager<TT, ST> {
    fn default() -> Self {
        TextureManager {
            textures: FnvHashMap::default(),
            sprites: FnvHashMap::default(),
        }
    }
}

impl<TT: TextureType, ST: SpriteType> TextureManager<TT, ST> {
    pub(self) fn load(
        &mut self,
        texture_type: TT,
        sprites: FnvHashMap<ST, SpriteDescriptor>,
    ) -> Result<(), TextureError> {
        if self.textures.get(&texture_type).is_none() {
            let img = image_loader::load(texture_type.get_path());
            let texture_id = Self::generate_texture(img.width, img.height, &img.data);

            self.textures.insert(texture_type, texture_id);

            for entry in sprites {
                self.sprites.insert(
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
                );
            }
            Ok(())
        } else {
            Err(TextureError::AlreadyLoaded)
        }
    }

    pub fn create(&mut self, texture_type: TT) -> TextureLoader<TT, ST> {
        TextureLoader {
            texture_manager: self,
            texture_type: texture_type,
            sprites: FnvHashMap::default(),
        }
    }

    pub fn destroy(&mut self, texture_type: TT) -> Result<(), TextureError> {
        match self.textures.remove_entry(&texture_type) {
            Some(texture) => {
                Self::destroy_texture(texture.1);

                let key_to_remove: Vec<ST> = self
                    .sprites
                    .iter()
                    .filter(|entry| entry.1.texture_id == texture.1)
                    .map(|(k, _)| k.clone())
                    .collect();
                for key in key_to_remove {
                    self.sprites.remove(&key);
                }

                self.textures.remove(&texture_type);
                Ok(())
            }
            _ => Err(TextureError::NotLoaded),
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

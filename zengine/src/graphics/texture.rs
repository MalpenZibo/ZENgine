use crate::assets::image_loader;
use crate::core::Resource;
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
    pub descriptor: SpriteDescriptor,
    pub texture_id: u32,
}

#[derive(Resource)]
pub struct TextureManager<TT: TextureType, ST: SpriteType> {
    textures: FnvHashMap<TT, u32>,
    sprites: FnvHashMap<ST, SpriteHandle>,
}

pub enum LoadError {
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
    pub fn load(
        &mut self,
        texture_type: TT,
        sprite_sheet: SpriteSheet<ST>,
    ) -> Result<(), LoadError> {
        if self.textures.get(&texture_type).is_none() {
            let img = image_loader::load(texture_type.get_path());
            let texture_id = Self::generate_texture(img.width, img.height, &img.data);

            for entry in sprite_sheet {
                self.sprites.insert(
                    entry.0,
                    SpriteHandle {
                        descriptor: entry.1,
                        texture_id,
                    },
                );
            }
            Ok(())
        } else {
            Err(LoadError::AlreadyLoaded)
        }
    }

    pub fn unload(&mut self, texture_type: TT) -> Result<(), LoadError> {
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
                Ok(())
            }
            _ => Err(LoadError::NotLoaded),
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

/*
pub struct Texture {
    width: u32,
    height: u32,

    texture_id: u32,
}

impl Texture {
    pub fn new(image_name: &str) -> Texture {
        let img = image_loader::load(image_name);

        let mut t = Texture {
            width: img.width,
            height: img.height,

            texture_id: 0,
        };

        unsafe {
            gl::GenTextures(1, &mut t.texture_id);
            gl::BindTexture(gl::TEXTURE_2D, t.texture_id);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                LEVEL,
                gl::RGBA as i32,
                t.width as i32,
                t.height as i32,
                BORDER,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                img.data.as_ptr() as *const gl::types::GLvoid,
            );

            gl::GenerateMipmap(gl::TEXTURE_2D);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        t
    }

    pub fn activate(&self) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture_id);
        }
    }
}*/

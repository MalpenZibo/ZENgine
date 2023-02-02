use std::ops::Deref;

use zengine_asset::AssetLoader;
use zengine_macro::Asset;

#[derive(Asset, Debug)]
pub struct Font(pub(crate) fontdue::Font);

impl Deref for Font {
    type Target = fontdue::Font;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Font {
    pub fn try_from_bytes(font_data: Vec<u8>) -> Self {
        let font = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default())
            .expect("Unable to load Font");

        Font(font)
    }
}

#[derive(Default, Debug)]
pub(crate) struct FontLoader;

impl AssetLoader for FontLoader {
    fn extension(&self) -> &[&str] {
        &["ttf", "otf"]
    }

    fn load(&self, data: Vec<u8>, context: &mut zengine_asset::LoaderContext) {
        let font = Font::try_from_bytes(data);

        context.set_asset(font);
    }
}

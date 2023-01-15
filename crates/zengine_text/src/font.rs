use ab_glyph::{FontArc, FontVec};
use glyph_brush_layout::FontId;
use zengine_asset::AssetLoader;
use zengine_macro::Asset;

#[derive(Asset, Debug)]
pub struct Font {
    pub font: FontArc,
    pub font_id: Option<FontId>,
}

impl Font {
    pub fn try_from_bytes(font_data: Vec<u8>) -> Self {
        let font = FontVec::try_from_vec(font_data).expect("Unable to load Font");
        let font = FontArc::new(font);
        Font {
            font,
            font_id: None,
        }
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

use glam::Vec2;
use zengine_asset::Handle;
use zengine_graphic::Color;
use zengine_macro::Component;

use crate::Font;

#[derive(Component, Debug)]
pub struct Text {
    pub sections: Vec<TextSection>,
    pub bounds: Vec2,

    pub style: TextStyle,

    pub alignment: TextAlignment,
}

#[derive(Debug)]
pub struct TextStyle {
    pub font: Handle<Font>,
    pub font_size: f32,
    pub color: Color,
}

#[derive(Debug)]
pub struct TextSection {
    pub value: String,
    pub style: Option<TextStyle>,
}

#[derive(Debug)]
pub struct TextAlignment {
    pub vertical: VerticalAlign,
    pub horizontal: HorizontalAlign,
}

impl TextAlignment {
    /// A [`TextAlignment`] set to the top-left.
    pub const TOP_LEFT: Self = TextAlignment {
        vertical: VerticalAlign::Top,
        horizontal: HorizontalAlign::Left,
    };

    /// A [`TextAlignment`] set to the top-center.
    pub const TOP_CENTER: Self = TextAlignment {
        vertical: VerticalAlign::Top,
        horizontal: HorizontalAlign::Center,
    };

    /// A [`TextAlignment`] set to the top-right.
    pub const TOP_RIGHT: Self = TextAlignment {
        vertical: VerticalAlign::Top,
        horizontal: HorizontalAlign::Right,
    };

    /// A [`TextAlignment`] set to center the center-left.
    pub const CENTER_LEFT: Self = TextAlignment {
        vertical: VerticalAlign::Center,
        horizontal: HorizontalAlign::Left,
    };

    /// A [`TextAlignment`] set to center on both axes.
    pub const CENTER: Self = TextAlignment {
        vertical: VerticalAlign::Center,
        horizontal: HorizontalAlign::Center,
    };

    /// A [`TextAlignment`] set to the center-right.
    pub const CENTER_RIGHT: Self = TextAlignment {
        vertical: VerticalAlign::Center,
        horizontal: HorizontalAlign::Right,
    };

    /// A [`TextAlignment`] set to the bottom-left.
    pub const BOTTOM_LEFT: Self = TextAlignment {
        vertical: VerticalAlign::Bottom,
        horizontal: HorizontalAlign::Left,
    };

    /// A [`TextAlignment`] set to the bottom-center.
    pub const BOTTOM_CENTER: Self = TextAlignment {
        vertical: VerticalAlign::Bottom,
        horizontal: HorizontalAlign::Center,
    };

    /// A [`TextAlignment`] set to the bottom-right.
    pub const BOTTOM_RIGHT: Self = TextAlignment {
        vertical: VerticalAlign::Bottom,
        horizontal: HorizontalAlign::Right,
    };
}

impl Default for TextAlignment {
    fn default() -> Self {
        Self::TOP_LEFT
    }
}

#[derive(Debug)]
pub enum HorizontalAlign {
    /// Leftmost character is immediately to the right of the render position.<br/>
    /// Bounds start from the render position and advance rightwards.
    Left,
    /// Leftmost & rightmost characters are equidistant to the render position.<br/>
    /// Bounds start from the render position and advance equally left & right.
    Center,
    /// Rightmost character is immediately to the left of the render position.<br/>
    /// Bounds start from the render position and advance leftwards.
    Right,
}

#[derive(Debug)]
pub enum VerticalAlign {
    /// Characters/bounds start underneath the render position and progress downwards.
    Top,
    /// Characters/bounds center at the render position and progress outward equally.
    Center,
    /// Characters/bounds start above the render position and progress upward.
    Bottom,
}

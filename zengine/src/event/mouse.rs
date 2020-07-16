use serde::Deserialize;

#[derive(Eq, PartialEq, Hash, Debug, Deserialize)]
#[repr(u8)]
pub enum MouseButton {
    Left = sdl2::mouse::MouseButton::Left as u8,
    Middle = sdl2::mouse::MouseButton::Middle as u8,
    Right = sdl2::mouse::MouseButton::Right as u8,
}

impl MouseButton {
    pub fn from_sdl_button(button: &sdl2::mouse::MouseButton) -> MouseButton {
        match unsafe { std::mem::transmute(*button as u8) } {
            Some(button) => button,
            None => panic!("Cannot convert number {} to `MouseButton`", *(button) as u8),
        }
    }
}

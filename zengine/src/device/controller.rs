use serde::Deserialize;

#[derive(Eq, PartialEq, Hash, Debug, Deserialize, Clone)]
#[repr(i32)]
pub enum ControllerButton {
    A = sdl2::controller::Button::A as i32,
    B = sdl2::controller::Button::B as i32,
    X = sdl2::controller::Button::X as i32,
    Y = sdl2::controller::Button::Y as i32,
    Back = sdl2::controller::Button::Back as i32,
    Guide = sdl2::controller::Button::Guide as i32,
    Start = sdl2::controller::Button::Start as i32,
    LeftStick = sdl2::controller::Button::LeftStick as i32,
    RightStick = sdl2::controller::Button::RightStick as i32,
    LeftShoulder = sdl2::controller::Button::LeftShoulder as i32,
    RightShoulder = sdl2::controller::Button::RightShoulder as i32,
    DPadUp = sdl2::controller::Button::DPadUp as i32,
    DPadDown = sdl2::controller::Button::DPadDown as i32,
    DPadLeft = sdl2::controller::Button::DPadLeft as i32,
    DPadRight = sdl2::controller::Button::DPadRight as i32,
}

impl ControllerButton {
    pub fn from_sdl_button(button: sdl2::controller::Button) -> ControllerButton {
        match unsafe { std::mem::transmute(button as i32) } {
            Some(button) => button,
            None => panic!(
                "Cannot convert number {} to `ControllerButton`",
                (button) as i32
            ),
        }
    }
}

#[derive(Eq, PartialEq, Hash, Debug, Deserialize, Clone)]
pub enum Which {
    Left,
    Right,
}

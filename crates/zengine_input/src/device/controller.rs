use std::ops::Deref;

use serde::Deserialize;

#[derive(Eq, PartialEq, Hash, Debug, Deserialize, Clone)]
//#[repr(i32)]
pub struct ControllerButton(pub gilrs::Button);

impl Deref for ControllerButton {
    type Target = gilrs::Button;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// impl ControllerButton {
//     pub fn from_sdl_button(button: sdl2::controller::Button) -> ControllerButton {
//         match unsafe { std::mem::transmute(button as i32) } {
//             Some(button) => button,
//             None => panic!(
//                 "Cannot convert number {} to `ControllerButton`",
//                 (button) as i32
//             ),
//         }
//     }
// }

#[derive(Eq, PartialEq, Hash, Debug, Deserialize, Clone)]
pub enum Which {
    Left,
    Right,
}

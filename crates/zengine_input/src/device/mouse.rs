use std::ops::Deref;

use serde::Deserialize;

#[derive(Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct MouseButton(pub winit::event::MouseButton);

impl Deref for MouseButton {
    type Target = winit::event::MouseButton;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

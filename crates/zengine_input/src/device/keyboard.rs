use std::ops::Deref;

use serde::Deserialize;

pub type Key = winit::event::VirtualKeyCode;

// #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Deserialize)]
// pub struct Key(pub winit::event::VirtualKeyCode);

// impl Deref for Key {
//     type Target = winit::event::VirtualKeyCode;
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

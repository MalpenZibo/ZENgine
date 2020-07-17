use crate::event::controller::ControllerButton;
use crate::event::controller::Which;
use crate::event::keyboard::Key;
use crate::event::mouse::MouseButton;
use serde::Deserialize;

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub enum Axis {
    X,
    Y,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub enum Input {
    Keyboard {
        key: Key,
    },
    MouseMotion {
        axis: Axis,
    },
    MouseWheel {
        axis: Axis,
    },
    MouseButton {
        button: MouseButton,
    },
    ControllerStick {
        device_id: u32,
        which: Which,
        axis: Axis,
    },
    ControllerTrigger {
        device_id: u32,
        which: Which,
    },
    ControllerButton {
        device_id: u32,
        button: ControllerButton,
    },
}

pub struct InputEvent {
    pub input: Input,
    pub value: f32,
}

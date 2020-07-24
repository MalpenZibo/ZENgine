use crate::device::controller::ControllerButton;
use crate::device::controller::Which;
use crate::device::keyboard::Key;
use crate::device::mouse::MouseButton;
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

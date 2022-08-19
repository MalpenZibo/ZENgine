use crate::device::{
    Key, MouseButton, {ControllerButton, Which},
};
use serde::Deserialize;

#[derive(Debug, Deserialize, Eq, PartialEq, Clone)]
pub enum Axis {
    X,
    Y,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Clone)]
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

#[derive(Debug)]
pub struct InputEvent {
    pub input: Input,
    pub value: f32,
}

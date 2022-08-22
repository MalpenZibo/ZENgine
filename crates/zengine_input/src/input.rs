use crate::device::{
    Key, MouseButton, {ControllerButton, Which},
};
use gilrs::GamepadId;
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
        device_id: GamepadId,
        which: Which,
        axis: Axis,
    },
    ControllerTrigger {
        device_id: GamepadId,
        which: Which,
    },
    ControllerButton {
        device_id: GamepadId,
        button: ControllerButton,
    },
}

#[derive(Debug)]
pub struct InputEvent {
    pub input: Input,
    pub value: f32,
}

use crate::device::{
    Key, MouseButton, {ControllerButton, Which},
};
use serde::Deserialize;

/// Rappresent an Axis
#[derive(Debug, Deserialize, Eq, PartialEq, Clone)]
pub enum Axis {
    /// X axis
    X,
    /// Y axis
    Y,
}

/// Rappresent an Input from the User
#[derive(Debug, Deserialize, Eq, PartialEq, Clone)]
pub enum Input {
    /// Triggered when the user press a keyboard key
    Keyboard {
        /// indicates which keyboard key has been pressed
        key: Key,
    },
    /// Triggered when the user moves the mouse
    MouseMotion {
        /// indicates the event motion axis
        axis: Axis,
    },
    /// Triggered when the user uses the mouse wheel
    MouseWheel {
        /// indicates the event wheel axis
        axis: Axis,
    },
    /// Triggered when the user press a mouse button
    MouseButton {
        /// indicate which button has been pressed
        button: MouseButton,
    },
    // Triggered when the user uses a gamepad stick
    ControllerStick {
        /// gamepad device id
        device_id: usize,
        /// indicates which stick as been used
        which: Which,
        /// indicates the event motion axis
        axis: Axis,
    },
    /// Triggered when the user uses a gamepad trigger
    ControllerTrigger {
        /// gamepad device id
        device_id: usize,
        /// indicates which trigger as been used
        which: Which,
    },
    /// Triggered when the user uses a gamepad button
    ControllerButton {
        /// gamepad device id
        device_id: usize,
        /// indicate which gamepad button has been pressed
        button: ControllerButton,
    },
    /// Triggered then the user touch the screen (only on supported platform)
    Touch {
        /// indixate the axis of the touch event
        axis: Axis,
    },
}

/// Rappresent an input event composed by the input type and its value
#[derive(Debug)]
pub struct InputEvent {
    pub input: Input,
    pub value: f32,
}

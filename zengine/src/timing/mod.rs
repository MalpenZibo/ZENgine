mod timing_system;
pub use self::timing_system::TimingSystem;

use crate::core::Resource;
use std::time::Duration;

#[derive(Debug)]
pub struct FrameLimiter {
    frame_duration: Duration,
}

impl FrameLimiter {
    pub fn new(fps: u32) -> Self {
        FrameLimiter {
            frame_duration: Duration::from_secs(1) / fps,
        }
    }
}

impl Default for FrameLimiter {
    fn default() -> Self {
        FrameLimiter {
            frame_duration: Duration::from_secs(1) / 60,
        }
    }
}

#[derive(Resource, Debug)]
pub struct Time {
    pub delta: Duration,
}

impl Default for Time {
    fn default() -> Self {
        Time {
            delta: Duration::from_secs(1),
        }
    }
}

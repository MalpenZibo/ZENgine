use instant::Instant;
use log::trace;
use std::thread::sleep;
use zengine_ecs::system::{Local, ResMut};

use std::time::Duration;
use zengine_macro::Resource;

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

#[derive(Debug)]
pub struct SystemInstant(Instant);
impl Default for SystemInstant {
    fn default() -> Self {
        SystemInstant(Instant::now())
    }
}

pub fn timing_system(limiter: Option<FrameLimiter>) -> impl Fn(ResMut<Time>, Local<SystemInstant>) {
    move |mut time: ResMut<Time>, last_call: Local<SystemInstant>| {
        let mut finish = Instant::now();
        let mut elapsed = finish - last_call.0;

        if let Some(limiter) = &limiter {
            if elapsed < limiter.frame_duration {
                sleep(limiter.frame_duration - elapsed);
                finish = Instant::now();
                elapsed += finish - last_call.0;
            }
        }

        time.delta = elapsed;
        last_call.0 = finish;

        trace!("time: {:?}", time);
    }
}

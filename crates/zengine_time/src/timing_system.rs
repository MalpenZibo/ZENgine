use log::trace;
use std::thread::sleep;
use std::time::Instant;
use zengine_ecs::system_parameter::{Local, ResMut};

use crate::{FrameLimiter, Time};

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

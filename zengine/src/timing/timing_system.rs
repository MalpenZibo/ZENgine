use crate::core::system::System;
use crate::core::system::Write;
use crate::core::Store;
use crate::timing::{FrameLimiter, Time};
use log::trace;
use std::thread::sleep;
use std::time::Instant;

#[derive(Debug)]
pub struct TimingSystem {
    last_call: Instant,
    limiter: Option<FrameLimiter>,
}

impl Default for TimingSystem {
    fn default() -> Self {
        TimingSystem {
            last_call: Instant::now(),
            limiter: None,
        }
    }
}

impl TimingSystem {
    pub fn with_limiter(mut self, limiter: FrameLimiter) -> Self {
        self.limiter = Some(limiter);

        self
    }
}

impl<'a> System<'a> for TimingSystem {
    type Data = Write<'a, Time>;

    fn init(&mut self, _store: &mut Store) {
        self.last_call = Instant::now();
    }

    fn run(&mut self, mut data: Self::Data) {
        let mut finish = Instant::now();
        let mut elapsed = finish - self.last_call;

        if let Some(limiter) = &self.limiter {
            if elapsed < limiter.frame_duration {
                sleep(limiter.frame_duration - elapsed);
                finish = Instant::now();
                elapsed += finish - self.last_call;
            }
        }

        data.delta = elapsed;
        self.last_call = finish;

        trace!("time: {:?}", data);
    }
}

use crate::core::system::System;
use crate::core::system::Write;
use crate::core::Resource;
use crate::core::Store;
use std::thread::sleep;
use std::time::{Duration, Instant};

#[derive(Debug, Default)]
pub struct Time {
    delta: f32,
}
impl Resource for Time {}

#[derive(Debug)]
pub struct FrameLimiterSystem {
    fps: u32,
    last_call: Instant,
    frame_duration: Duration,
}

impl FrameLimiterSystem {
    pub fn new(fps: u32) -> Self {
        FrameLimiterSystem {
            fps: fps,
            last_call: Instant::now(),
            frame_duration: Duration::from_secs(1) / fps,
        }
    }
}

impl Default for FrameLimiterSystem {
    fn default() -> Self {
        FrameLimiterSystem {
            fps: 60,
            last_call: Instant::now(),
            frame_duration: Duration::from_secs(1) / 60,
        }
    }
}

impl<'a> System<'a> for FrameLimiterSystem {
    type Data = Write<'a, Time>;

    fn init(&mut self, store: &mut Store) {
        self.last_call = Instant::now();
    }

    fn run(&mut self, mut data: Self::Data) {
        let elapsed = Instant::now() - self.last_call;

        if elapsed < self.frame_duration {
            //println!("sleep for: {:?}", self.frame_duration - elapsed);
            sleep(self.frame_duration - elapsed);
        }

        let finish = Instant::now();
        data.delta = finish.duration_since(self.last_call).as_secs_f32();
        self.last_call = finish;

        /*
        println!(
            "delta: {:?}, frame_duration: {:?}, elapsed: {:?}",
            data, self.frame_duration, elapsed
        );*/
    }
}

use instant::Instant;
use log::trace;
use std::thread::sleep;
use zengine_ecs::system::{EventStream, Local, ResMut};
use zengine_engine::{EngineEvent, Module, StageLabel};

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

pub struct TimeModule(pub Option<FrameLimiter>);
impl Module for TimeModule {
    fn init(self, engine: &mut zengine_engine::Engine) {
        engine.add_system_into_stage(timing_system(self.0), StageLabel::PreUpdate);
    }
}

fn timing_system(
    limiter: Option<FrameLimiter>,
) -> impl Fn(EventStream<EngineEvent>, ResMut<Time>, Local<SystemInstant>) {
    move |engine_event: EventStream<EngineEvent>,
          mut time: ResMut<Time>,
          last_call: Local<SystemInstant>| {
        if engine_event.read().last() == Some(&EngineEvent::Resumed) {
            last_call.0 = Instant::now();
        }

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

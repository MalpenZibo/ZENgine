use instant::Instant;
use log::trace;
use std::thread::sleep;
use zengine_ecs::system::{EventStream, Local, ResMut};
use zengine_engine::{EngineEvent, Module, Stage};

use std::time::Duration;
use zengine_macro::Resource;

/// A struct that rappresent a frame limiter
#[derive(Debug)]
pub struct FrameLimiter {
    frame_duration: Duration,
}

impl FrameLimiter {
    /// Create a framelimiter from a target frame per second
    pub fn new(fps: u32) -> Self {
        FrameLimiter {
            frame_duration: Duration::from_secs(1) / fps,
        }
    }
}

impl Default for FrameLimiter {
    /// Returns a default frame limiter of 60 frames per second
    fn default() -> Self {
        FrameLimiter {
            frame_duration: Duration::from_secs(1) / 60,
        }
    }
}

/// A [Resource](zengine_ecs::Resource) that contains the time elapsed between frames
#[derive(Resource, Debug)]
pub struct Time {
    /// Delta time between each frame
    delta: Duration,
}

impl Default for Time {
    fn default() -> Self {
        Time {
            delta: Duration::from_secs(1),
        }
    }
}

impl Time {
    /// Returns how much time has advanced since the cycle, as a [`Duration`].
    pub fn delta(&self) -> Duration {
        self.delta
    }
}

#[derive(Debug)]
struct SystemInstant(Instant);
impl Default for SystemInstant {
    fn default() -> Self {
        SystemInstant(Instant::now())
    }
}

/// Adds timing suport to the engine
///
/// This module add a [Time] resource that measure
/// the time passed between each frame
pub struct TimeModule(
    /// Optional [FrameLimiter] to add to the engine
    pub Option<FrameLimiter>,
);
impl Module for TimeModule {
    fn init(self, engine: &mut zengine_engine::Engine) {
        engine.add_system_into_stage(timing_system(self.0), Stage::PreUpdate);
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

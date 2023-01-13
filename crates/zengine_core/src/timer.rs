use instant::Duration;
use zengine_macro::{Component, Resource};

use crate::Time;

/// Tracks elapsed time. Enters the finished state once `duration` is reached.
#[derive(Resource, Component, Debug)]
pub struct Timer {
    elapsed: Duration,
    duration: Duration,
    finished: bool,
}

impl Timer {
    /// Creates a new timer with a given duration.
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            elapsed: Duration::default(),
            finished: false,
        }
    }

    /// Advance the timer.
    pub fn tick(&mut self, time: &Time) {
        self.elapsed += time.delta();
        self.finished = self.elapsed >= self.duration;
    }

    /// Returns `true` if the timer has reached its duration
    pub fn finished(&self) -> bool {
        self.finished
    }
}

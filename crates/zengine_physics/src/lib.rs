mod collision;
mod tracer;

pub use collision::*;
pub use tracer::*;
use zengine_engine::{
    schedule::default_schedule::{PostUpdate, Render},
    Engine, Module,
};

/// Adds a simple collision system to the engine.
#[derive(Default, Debug)]
pub struct CollisionModule {
    pub with_tracer: bool,
}

impl CollisionModule {
    pub fn with_tracer() -> Self {
        CollisionModule { with_tracer: true }
    }
}

impl Module for CollisionModule {
    fn init(self, engine: &mut Engine) {
        engine.add_system_into_schedule(collision_system, PostUpdate);

        if self.with_tracer {
            engine
                .add_startup_system(setup_trace_render)
                .add_system_into_schedule(collision_system, Render)
                .add_system_into_schedule(collision_tracer, Render);
        }
    }
}

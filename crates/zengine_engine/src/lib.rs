use std::collections::HashMap;

use simplelog::{Config, LevelFilter, SimpleLogger, TermLogger, TerminalMode};
use zengine_ecs::{
    system::{IntoSystem, System},
    system_parameter::SystemParam,
    world::World,
};

pub use log;

#[derive(Hash, Eq, PartialEq)]
pub enum StageLabel {
    Startup,
    Update,
    Render,
    PostRender,
}

#[derive(Default)]
pub struct Stage {
    systems: Vec<Box<dyn System>>,
}

impl Stage {
    pub fn init(&mut self, world: &mut World) {
        for s in self.systems.iter_mut() {
            s.init(world);
        }
    }

    pub fn run(&mut self, world: &World) {
        for s in self.systems.iter_mut() {
            s.run(world);
        }
    }

    pub fn apply(&mut self, world: &mut World) {
        for s in self.systems.iter_mut() {
            s.apply(world);
        }
    }

    pub fn run_and_apply(&mut self, world: &mut World) {
        for s in self.systems.iter_mut() {
            s.run(world);
            s.apply(world);
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum EngineEvent {
    Quit,
}

pub struct Engine {
    stages: HashMap<StageLabel, Stage>,
    stage_order: Vec<StageLabel>,
    world: World,
}

impl Default for Engine {
    fn default() -> Self {
        Engine {
            stages: HashMap::from([
                (StageLabel::Startup, Stage::default()),
                (StageLabel::Update, Stage::default()),
                (StageLabel::Render, Stage::default()),
                (StageLabel::PostRender, Stage::default()),
            ]),
            stage_order: vec![
                StageLabel::Startup,
                StageLabel::Update,
                StageLabel::Render,
                StageLabel::PostRender,
            ],
            world: World::default(),
        }
    }
}

impl Engine {
    pub fn init_logger(level_filter: LevelFilter) {
        if TermLogger::init(level_filter, Config::default(), TerminalMode::Mixed).is_err() {
            SimpleLogger::init(level_filter, Config::default())
                .expect("An error occurred on logger initialization")
        }

        log_panics::init();
    }

    pub fn add_system<Params: SystemParam + 'static, I: IntoSystem<Params> + 'static>(
        self,
        system: I,
    ) -> Self {
        self.add_system_into_stage(system, StageLabel::Update)
    }

    pub fn add_startup_system<Params: SystemParam + 'static, I: IntoSystem<Params> + 'static>(
        self,
        system: I,
    ) -> Self {
        self.add_system_into_stage(system, StageLabel::Startup)
    }

    pub fn add_system_into_stage<Params: SystemParam + 'static, I: IntoSystem<Params> + 'static>(
        mut self,
        system: I,
        stage: StageLabel,
    ) -> Self {
        if let Some(stage) = self.stages.get_mut(&stage) {
            stage.systems.push(Box::new(system.into_system()));
        }

        self
    }

    pub fn run(mut self) {
        self.world.create_event_handler::<EngineEvent>();

        let mut stages: Vec<Stage> = self
            .stage_order
            .iter()
            .map(|stage_label| self.stages.remove(stage_label).unwrap())
            .collect();

        for stage in stages.iter_mut() {
            stage.init(&mut self.world);
        }

        let mut startup_stage = stages.remove(0);
        startup_stage.run_and_apply(&mut self.world);

        'main_loop: loop {
            for stage in stages.iter_mut() {
                stage.run(&self.world);
            }

            for stage in stages.iter_mut() {
                stage.apply(&mut self.world);
            }

            {
                let engine_event = self.world.get_event_handler::<EngineEvent>().unwrap();
                if engine_event
                    .read_last()
                    .map_or_else(|| false, |e| e == &EngineEvent::Quit)
                {
                    break 'main_loop;
                }
            }
        }
    }
}

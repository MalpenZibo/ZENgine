use std::{any::Any, collections::HashMap};

use zengine_ecs::{
    system::{IntoSystem, System, SystemParam},
    World,
};

pub use log;

pub trait Module {
    fn init(self, engine: &mut Engine);
}

#[derive(Hash, Eq, PartialEq)]
pub enum StageLabel {
    Startup,
    PreUpdate,
    Update,
    PostUpdate,
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

#[derive(Debug, PartialEq, Eq)]
pub enum EngineEvent {
    Quit,
}

pub struct Engine {
    stages: HashMap<StageLabel, Stage>,
    stage_order: Vec<StageLabel>,
    running_stages: Vec<Stage>,
    pub world: World,
    runner: Box<dyn Fn(Engine)>,
}

impl Default for Engine {
    fn default() -> Self {
        Engine {
            stages: HashMap::from([
                (StageLabel::Startup, Stage::default()),
                (StageLabel::PreUpdate, Stage::default()),
                (StageLabel::Update, Stage::default()),
                (StageLabel::PostUpdate, Stage::default()),
                (StageLabel::Render, Stage::default()),
                (StageLabel::PostRender, Stage::default()),
            ]),
            stage_order: vec![
                StageLabel::Startup,
                StageLabel::PreUpdate,
                StageLabel::Update,
                StageLabel::PostUpdate,
                StageLabel::Render,
                StageLabel::PostRender,
            ],
            running_stages: Vec::default(),
            world: World::default(),
            runner: Box::new(default_runner),
        }
    }
}

fn default_runner(mut engine: Engine) {
    loop {
        if engine.update() {
            break;
        }
    }
}

impl Engine {
    pub fn init_logger(level: log::Level) {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init_with_level(level).expect("Couldn't initialize logger");
            } else {
                use simplelog::{Config, SimpleLogger, TermLogger, TerminalMode};
                let level_filter = level.to_level_filter();
                if TermLogger::init(level_filter, Config::default(), TerminalMode::Mixed).is_err() {
                    SimpleLogger::init(level_filter, Config::default())
                        .expect("Couldn't initialize logger")
                }

                log_panics::init();
            }
        }
    }

    pub fn add_system<Params: SystemParam + Any, I: IntoSystem<Params> + Any>(
        &mut self,
        system: I,
    ) -> &mut Self {
        self.add_system_into_stage(system, StageLabel::Update)
    }

    pub fn add_startup_system<Params: SystemParam + Any, I: IntoSystem<Params> + Any>(
        &mut self,
        system: I,
    ) -> &mut Self {
        self.add_system_into_stage(system, StageLabel::Startup)
    }

    pub fn add_system_into_stage<Params: SystemParam + Any, I: IntoSystem<Params> + Any>(
        &mut self,
        system: I,
        stage: StageLabel,
    ) -> &mut Self {
        if let Some(stage) = self.stages.get_mut(&stage) {
            stage.systems.push(Box::new(system.into_system()));
        }

        self
    }

    pub fn add_module(&mut self, module: impl Module) -> &mut Self {
        module.init(self);

        self
    }

    pub fn set_runner<F: Fn(Engine) + 'static>(&mut self, runner: F) -> &mut Self {
        self.runner = Box::new(runner);
        self
    }

    pub fn startup(&mut self) {
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

        self.running_stages = stages;
    }

    pub fn update(&mut self) -> bool {
        for stage in self.running_stages.iter_mut() {
            stage.run(&self.world);
        }

        for stage in self.running_stages.iter_mut() {
            stage.apply(&mut self.world);
        }

        {
            let engine_event = self.world.get_event_handler::<EngineEvent>().unwrap();
            if engine_event
                .read_last()
                .map_or_else(|| false, |e| e == &EngineEvent::Quit)
            {
                return true;
            }
        }

        false
    }

    pub fn run(&mut self) {
        self.world.create_event_handler::<EngineEvent>();

        let mut app = std::mem::take(self);
        let runner = std::mem::replace(&mut app.runner, Box::new(default_runner));

        (runner)(app);
    }
}

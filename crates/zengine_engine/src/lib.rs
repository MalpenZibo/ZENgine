use std::collections::HashMap;

use zengine_ecs::{
    system::{IntoSystem, System},
    system_parameter::SystemParam,
    world::World,
};

#[derive(Hash, Eq, PartialEq)]
pub enum StageLabel {
    Startup,
    Update,
    Render,
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
            ]),
            stage_order: vec![StageLabel::Startup, StageLabel::Update, StageLabel::Render],
            world: World::default(),
        }
    }
}

impl Engine {
    fn add_system<Params: SystemParam + 'static, I: IntoSystem<Params> + 'static>(
        mut self,
        system: I,
    ) -> Self {
        self.add_system_into_stage(system, StageLabel::Update)
    }

    fn add_startup_system<Params: SystemParam + 'static, I: IntoSystem<Params> + 'static>(
        mut self,
        system: I,
    ) -> Self {
        self.add_system_into_stage(system, StageLabel::Startup)
    }

    fn add_system_into_stage<Params: SystemParam + 'static, I: IntoSystem<Params> + 'static>(
        mut self,
        system: I,
        stage: StageLabel,
    ) -> Self {
        if let Some(stage) = self.stages.get_mut(&stage) {
            stage.systems.push(Box::new(system.into_system()));
        }

        self
    }

    fn run(mut self) {
        let mut stages: Vec<Stage> = self
            .stage_order
            .iter()
            .map(|stage_label| self.stages.remove(stage_label).unwrap())
            .collect();

        for stage in stages.iter_mut() {
            stage.init(&mut self.world);
        }

        let mut startup_stage = stages.remove(0);
        startup_stage.run(&self.world);
        startup_stage.apply(&mut self.world);

        loop {
            for stage in stages.iter_mut() {
                stage.run(&self.world);
            }

            for stage in stages.iter_mut() {
                stage.apply(&mut self.world);
            }
        }
    }
}

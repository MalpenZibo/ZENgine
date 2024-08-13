use std::{any::TypeId, fmt::Debug};

use default_schedule::*;
use log::debug;
use zengine_ecs::{
    system::{BoxedSystem, IntoSystem, SystemFunction},
    World,
};

use crate::ScheduleLabel;

pub mod default_schedule {
    /// The possible stages in the engine pipeline
    use zengine_macro::ScheduleLabel;

    /// Statup stage, runs only one time when the engine start
    #[derive(ScheduleLabel, Copy, Clone, Debug)]
    pub struct Startup;

    /// Run just before the main update stage
    #[derive(ScheduleLabel, Copy, Clone, Debug)]
    pub struct PreUpdate;

    /// Main stage
    #[derive(ScheduleLabel, Copy, Clone, Debug)]
    pub struct Update;

    /// Run after the update stage
    #[derive(ScheduleLabel, Copy, Clone, Debug)]
    pub struct PostUpdate;

    /// Run before the render stage
    #[derive(ScheduleLabel, Copy, Clone, Debug)]
    pub struct PreRender;

    /// Render stage, draws the new state
    #[derive(ScheduleLabel, Copy, Clone, Debug)]
    pub struct Render;

    /// Run after the render stage
    #[derive(ScheduleLabel, Copy, Clone, Debug)]
    pub struct PostRender;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ScheduleLabelInternal(pub(crate) TypeId, pub(crate) &'static str);

pub struct Schedule {
    label: ScheduleLabelInternal,
    systems: Vec<BoxedSystem>,
    conditions: Vec<BoxedSystem>,
}

impl Debug for Schedule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Schedule")
            .field("label", &self.label.1)
            .field("systems", &self.systems.len())
            .field("conditions", &self.conditions.len())
            .finish()
    }
}

impl Schedule {
    pub fn new(label: impl ScheduleLabel) -> Self {
        Self {
            label: label.internal(),
            systems: Vec::new(),
            conditions: Vec::new(),
        }
    }

    pub fn add_system<Marker, S: IntoSystem<Marker>>(&mut self, system: S) {
        self.systems.push(system.into_system());
    }

    pub fn init(&mut self, world: &mut World) {
        for system in self.systems.iter_mut() {
            system.init(world);
        }
    }

    pub fn run(&mut self, world: &World) {
        for system in self.systems.iter_mut() {
            system.run(world);
        }
    }

    pub fn apply(&mut self, world: &mut World) {
        for system in self.systems.iter_mut() {
            system.apply(world);
        }
    }

    pub fn run_and_apply(&mut self, world: &mut World) {
        for system in self.systems.iter_mut() {
            system.run(world);
            system.apply(world);
        }
    }
}

#[derive(Debug)]
pub struct Schedules(Vec<Schedule>);

impl Default for Schedules {
    fn default() -> Self {
        Self(vec![
            Schedule::new(Startup),
            Schedule::new(PreUpdate),
            Schedule::new(Update),
            Schedule::new(PostUpdate),
            Schedule::new(PreRender),
            Schedule::new(Render),
            Schedule::new(PostRender),
        ])
    }
}

impl Schedules {
    pub fn add_system<Marker, S: IntoSystem<Marker>>(
        &mut self,
        label: impl ScheduleLabel,
        system: S,
    ) {
        let schedule = self
            .0
            .iter_mut()
            .find(|s| s.label == label.internal())
            .expect("Schedule not found");

        schedule.add_system(system);
    }

    pub fn startup(&mut self, world: &mut World) {
        debug!("Starting schedules");

        for schedule in self.0.iter_mut() {
            debug!("init schedule {:?}", schedule.label);
            schedule.init(world);
        }

        if let Some(index) = self.0.iter().position(|s| s.label == Startup.internal()) {
            let mut schedule = self.0.remove(index);

            debug!("Run and apply on startup schedule");
            schedule.run_and_apply(world);
        }
    }

    pub fn run(&mut self, world: &World) {
        for schedule in self.0.iter_mut() {
            schedule.run(world);
        }
    }

    pub fn apply(&mut self, world: &mut World) {
        for schedule in self.0.iter_mut() {
            schedule.apply(world);
        }
    }
}

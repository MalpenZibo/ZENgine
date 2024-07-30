use std::{
    any::{Any, TypeId},
    fmt::Debug,
};

use hashbrown::HashMap;
use zengine_ecs::system::{condition::Condition, IntoSystem, System, SystemParam};

#[derive(Debug)]
struct ScheduleLabelInternal(TypeId);

pub trait ScheduleLabel: Copy + Clone + 'static {
    fn internal(&self) -> ScheduleLabelInternal {
        ScheduleLabelInternal(TypeId::of::<Self>())
    }
}

#[derive(Default)]
pub struct Schedule {
    systems: Vec<Box<dyn System>>,
    conditions: Vec<Box<dyn Condition>>,
}

impl Debug for Schedule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Schedule").finish()
    }
}

impl Schedule {
    pub fn add_system<Params: SystemParam + Any, I: IntoSystem<Params> + Any>(
        &mut self,
        system: I,
    ) {
        self.systems.push(Box::new(system.into_system()));
    }
}

#[derive(Default, Debug)]
pub struct Schedules(HashMap<ScheduleLabelInternal, Schedule>);

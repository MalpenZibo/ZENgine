extern crate zengine;

use zengine::basic::platform::{EventPumpSystem, WindowSystem};
use zengine::basic::timing::{FrameLimiter, TimingSystem};
use zengine::core::event::EventStream;
use zengine::core::event::SubscriptionToken;
use zengine::core::system::Read;
use zengine::core::system::{ReadSet, WriteSet};
use zengine::core::Component;
use zengine::core::Scene;
use zengine::core::Store;
use zengine::core::System;
use zengine::core::Trans;
use zengine::log::{trace, LevelFilter};
use zengine::Engine;

fn main() {
    Engine::init_logger(LevelFilter::Warn);

    Engine::default()
        .with_system(EventPumpSystem::default())
        .with_system(System1::default())
        .with_system(System2::default())
        .with_system(WindowSystem::default())
        .with_system(TimingSystem::default().with_limiter(FrameLimiter::new(1)))
        .run(Game {
            execution_numer: 10,
        });
}

pub struct Game {
    execution_numer: u32,
}

impl Scene for Game {
    fn on_start(&mut self, store: &mut Store) {
        trace!("Game scene on start");

        store.build_entity().with(Component1 { data: 3 }).build();
        store
            .build_entity()
            .with(Component1 { data: 3 })
            .with(Component2 { data2: 5 })
            .build();
    }

    fn on_stop(&mut self, store: &mut Store) {
        trace!("Game scene on stop");
    }

    fn update(&mut self, store: &Store) -> Trans {
        match self.execution_numer {
            0 => Trans::None,
            _ => {
                self.execution_numer -= 1;
                Trans::None
            }
        }
    }
}

#[derive(Debug)]
pub struct Component1 {
    data: u32,
}

impl Component for Component1 {}

#[derive(Debug)]
pub struct Component2 {
    data2: u32,
}

impl Component for Component2 {}

#[derive(Debug)]
pub struct Component3 {
    data2: u32,
}

impl Component for Component3 {}

#[derive(Debug, Default)]
pub struct System1 {
    run_count: u32,
}

impl<'a> System<'a> for System1 {
    type Data = (ReadSet<'a, Component1>, WriteSet<'a, Component2>);

    fn init(&mut self, store: &mut Store) {
        trace!("setup system 1");
    }

    fn run(&mut self, data: Self::Data) {
        trace!("run {} system 1", self.run_count);

        let (c1, mut c2) = data; //unpack!(store, );

        for c in c2.values_mut() {
            c.data2 += 1;
        }

        trace!("c1 {:?}", c1);
        trace!("c2 {:?}", c2);

        self.run_count += 1;
    }

    fn dispose(&mut self, store: &mut Store) {
        trace!("dispose system 1");
    }
}

#[derive(Debug, Default)]
pub struct System2 {
    run_count: u32,
    token: Option<SubscriptionToken>,
}

impl<'a> System<'a> for System2 {
    type Data = (WriteSet<'a, Component2>, Read<'a, EventStream<Trans>>);

    fn init(&mut self, store: &mut Store) {
        trace!("setup system 2");

        self.token = Some(
            store
                .get_resource_mut::<EventStream<Trans>>()
                .unwrap()
                .subscribe(),
        );
    }

    fn run(&mut self, (set, stream): Self::Data) {
        trace!("run {} system 2", self.run_count);

        trace!("data 2: {:?}", set);
        trace!("data 2: {:?}", stream.read(&self.token.unwrap()));

        self.run_count += 1;
    }

    fn dispose(&mut self, store: &mut Store) {
        trace!("dispose system 2");
    }
}

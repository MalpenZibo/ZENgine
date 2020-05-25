#[macro_use]
extern crate zengine;

use zengine::core::system::{Data, ReadSet, WriteSet};
use zengine::core::Component;
use zengine::core::Scene;
use zengine::core::Store;
use zengine::core::System;
use zengine::core::Trans;
use zengine::Engine;

fn main() {
    Engine::default()
        .with_system(System1::default())
        .with_system(System2::default())
        .run(Game {
            execution_numer: 10,
        });
}

pub struct Game {
    execution_numer: u32,
}

impl Scene for Game {
    fn on_start(&mut self, store: &mut Store) {
        println!("Game scene on start");

        store.build_entity().with(Component1 { data: 3 }).build();
        store
            .build_entity()
            .with(Component1 { data: 3 })
            .with(Component2 { data2: 5 })
            .build();
    }

    fn on_stop(&mut self, store: &mut Store) {
        println!("Game scene on stop");
    }

    fn update(&mut self, store: &mut Store) -> Trans {
        match self.execution_numer {
            0 => Trans::Quit,
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

#[derive(Debug, Default)]
pub struct System1 {
    run_count: u32,
}

impl System for System1 {
    fn init(&mut self, store: &mut Store) {
        println!("setup system 1");
    }

    fn run(&mut self, store: &Store) {
        println!("run {} system 1", self.run_count);

        let (c1, mut c2) = unpack!(store, ReadSet<Component1>, WriteSet<Component2>);

        for c in c2.values_mut() {
            c.data2 += 1;
        }

        println!("c1 {:?}", c1);
        println!("c2 {:?}", c2);

        self.run_count += 1;
    }

    fn dispose(&mut self, store: &mut Store) {
        println!("dispose system 1");
    }
}

#[derive(Debug, Default)]
pub struct System2 {
    run_count: u32,
}

impl System for System2 {
    fn init(&mut self, store: &mut Store) {
        println!("setup system 2");
    }

    fn run(&mut self, store: &Store) {
        println!("run {} system 2", self.run_count);

        self.run_count += 1;
    }

    fn dispose(&mut self, store: &mut Store) {
        println!("dispose system 2");
    }
}

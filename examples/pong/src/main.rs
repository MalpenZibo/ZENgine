extern crate zengine;

use zengine::core::Scene;
use zengine::core::Store;
use zengine::core::System;
use zengine::core::Trans;
use zengine::EngineBuilder;

fn main() {
    /*zengine::engine::start(
        zengine::engine::EngineOption {
            title: String::from("PONG"),
            fullscreen: false,
            virtual_width: 1920,
            virtual_height: 1080,
            screen_width: 800,
            screen_height: 600
        }
    );*/

    let engine = EngineBuilder::default()
        .with_system(System1::default())
        .with_system(System2::default())
        .build();

    engine.run(Game {
        execution_numer: 10,
    })
}

pub struct Game {
    execution_numer: u32,
}

impl Scene for Game {
    fn on_start(&mut self, store: &mut Store) {
        println!("Game scene on start");
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

#[derive(Debug, Default)]
pub struct System1 {
    run_count: u32,
}

impl System for System1 {
    fn init(&mut self, store: &mut Store) {
        println!("setup system 1");
    }

    fn run(&mut self, store: &mut Store) {
        println!("run {} system 1", self.run_count);

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

    fn run(&mut self, store: &mut Store) {
        println!("run {} system 2", self.run_count);

        self.run_count += 1;
    }

    fn dispose(&mut self, store: &mut Store) {
        println!("dispose system 2");
    }
}

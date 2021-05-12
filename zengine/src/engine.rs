extern crate log_panics;

use crate::core::scene::AnyScene;
use crate::core::system::AnySystem;
use crate::core::Store;
use crate::core::System;
use crate::core::Trans;
use log::info;
use simplelog::{Config, LevelFilter, SimpleLogger, TermLogger, TerminalMode};

#[derive(Default)]
pub struct Engine {
    store: Store,
    systems: Vec<Box<dyn AnySystem>>,
}

impl Engine {
    pub fn init_logger(level_filter: LevelFilter) {
        if TermLogger::init(level_filter, Config::default(), TerminalMode::Mixed).is_err() {
            SimpleLogger::init(level_filter, Config::default())
                .expect("An error occurred on logger initialization")
        }

        log_panics::init();
    }

    pub fn with_system<S: for<'a> System<'a>>(mut self, system: S) -> Self {
        self.systems.push(Box::new(system));

        self
    }

    pub fn run<S: AnyScene + 'static>(mut self, mut scene: S) {
        info!("Engine Start");

        info!("Init Systems");
        for s in self.systems.iter_mut() {
            s.init(&mut self.store);
        }
        info!("Scene Start");
        scene.on_start(&mut self.store);

        'main_loop: loop {
            for s in self.systems.iter_mut() {
                s.run(&self.store);
            }
            if let Trans::Quit = scene.update(&self.store) {
                info!("Quit transaction received");
                break 'main_loop;
            }
        }

        info!("Scene Stop");
        scene.on_stop(&mut self.store);

        info!("Dispose Systems");
        for s in self.systems.iter_mut() {
            s.dispose(&mut self.store);
        }

        info!("Engine Stop");
    }
}

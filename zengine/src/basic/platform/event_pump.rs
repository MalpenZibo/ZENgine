extern crate sdl2;

use crate::basic::platform::resources::Platform;
use crate::core::event::EventStream;
use crate::core::system::System;
use crate::core::system::Write;
use crate::core::Store;
use crate::core::Trans;
use log::info;
use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::Sdl;

pub struct EventPumpSystem {
    consumable_sdl_context: Option<Sdl>,
    event_pump: EventPump,
}

impl Default for EventPumpSystem {
    fn default() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();

        EventPumpSystem {
            consumable_sdl_context: Some(sdl_context),
            event_pump: event_pump,
        }
    }
}

impl<'a> System<'a> for EventPumpSystem {
    type Data = Write<'a, EventStream<Trans>>;

    fn init(&mut self, store: &mut Store) {
        let context = self.consumable_sdl_context.take();

        store.insert_resource(Platform {
            context: context.unwrap(),
        });
    }

    fn run(&mut self, mut data: Self::Data) {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    info!("quit event sended");
                    data.publish(Trans::Quit);
                }
                _ => (),
            }
        }
    }
}

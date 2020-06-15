extern crate sdl2;

use crate::basic::platform::resources::Platform;
use crate::core::system::System;
use crate::core::Store;
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
    type Data = ();

    fn init(&mut self, store: &mut Store) {
        let context = self.consumable_sdl_context.take();

        store.insert_resource(Platform {
            context: context.unwrap(),
        });
    }

    fn run(&mut self, data: Self::Data) {
        for event in self.event_pump.poll_iter() {
            match event {
                _ => (),
            }
        }
    }
}

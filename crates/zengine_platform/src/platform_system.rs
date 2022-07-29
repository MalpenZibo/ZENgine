extern crate sdl2;

use std::fmt::Debug;

use log::{info, trace};
use rustc_hash::FxHashMap;
use sdl2::controller::GameController;
use sdl2::EventPump;
use zengine_ecs::command::Commands;
use zengine_ecs::world::UnsendableResource;

use crate::VideoSubsystemWrapper;

pub struct PlatformContext {
    pub event_pump: EventPump,
}

pub struct Controllers(pub FxHashMap<u32, (u32, GameController)>);

impl Debug for Controllers {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl UnsendableResource for Controllers {}

impl Debug for PlatformContext {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl UnsendableResource for PlatformContext {}

pub fn platform_startup(mut commands: Commands) {
    let sdl_context = sdl2::init().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();
    let controller_subsystem = sdl_context.game_controller().unwrap();

    let available = controller_subsystem
        .num_joysticks()
        .map_err(|e| format!("can't enumerate joysticks: {}", e))
        .unwrap();

    info!("{} joysticks available", available);

    let mut controller_index: u32 = 0;
    let controllers: FxHashMap<u32, (u32, GameController)> = (0..available)
        .filter_map(|id| {
            if controller_subsystem.is_game_controller(id) {
                controller_index += 1;

                trace!("Attempting to open controller {}", id);

                match controller_subsystem.open(id) {
                    Ok(c) => {
                        trace!("Success: opened {}", c.name());

                        Some((id, (controller_index, c)))
                    }
                    Err(e) => {
                        trace!("failed: {:?}", e);

                        None
                    }
                }
            } else {
                trace!("{} is not a game controller", id);
                None
            }
        })
        .collect();
    let video_subsystem = sdl_context.video().unwrap();

    commands.create_unsendable_resource(PlatformContext { event_pump });
    commands.create_unsendable_resource(Controllers(controllers));
    commands.create_unsendable_resource(VideoSubsystemWrapper(video_subsystem))
}

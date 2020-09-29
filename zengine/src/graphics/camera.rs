use crate::core::Component;
use crate::core::Entity;
use crate::core::Resource;

#[derive(Resource)]
pub struct ActiveCamera {
    pub entity_id: Entity,
}

#[derive(Component, Debug)]
pub struct Camera {
    pub width: u32,
    pub height: u32,
}

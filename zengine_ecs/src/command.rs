use std::any::TypeId;

use crate::{
    component::ComponentBundle,
    entity::{Entity, EntityGenerator},
    system_parameter::{SystemParam, SystemParamFetch},
    world::{Resource, World},
};

pub trait Command {
    fn exec(self, world: &mut World);
}

type CommandState = Vec<Box<dyn Command>>;

struct SpawnCommand<T: ComponentBundle> {
    entity: Entity,
    components: T,
}

impl<T: ComponentBundle> Command for SpawnCommand<T> {
    fn exec(self, world: &mut World) {
        world.add_component(self.entity, self.components);
    }
}

struct DespawnCommand {
    entity: Entity,
}

impl Command for DespawnCommand {
    fn exec(self, world: &mut World) {
        world.despawn(self.entity);
    }
}

struct CreateResourceCommand<T: Resource> {
    resource: T,
}

impl<T: Resource> Command for CreateResourceCommand<T> {
    fn exec(self, world: &mut World) {
        world.create_resource(self.resource);
    }
}

struct DestroyResourceCommand {
    resource_type: TypeId,
}

impl Command for DestroyResourceCommand {
    fn exec(self, world: &mut World) {
        world.destroy_resource_with_type_id(self.resource_type);
    }
}

pub struct Commands<'a> {
    queue: &'a mut CommandState,
    entities: &'a EntityGenerator,
}

impl<'a> Commands<'a> {
    pub fn spawn<T: ComponentBundle + 'static>(&mut self, component_bundle: T) -> Entity {
        let entity = self.entities.generate();
        self.queue.push(Box::new(SpawnCommand {
            entity,
            components: component_bundle,
        }));

        entity
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.queue.push(Box::new(DespawnCommand { entity }))
    }

    pub fn create_resource<T: Resource>(&mut self, resource: T) {
        self.queue
            .push(Box::new(CreateResourceCommand { resource }))
    }

    pub fn destroy_resource<T: Resource>(&mut self) {
        self.queue.push(Box::new(DestroyResourceCommand {
            resource_type: TypeId::of::<T>(),
        }))
    }
}

impl<'a> SystemParamFetch<'a> for CommandState {
    type Item = Commands<'a>;

    fn fetch(&'a mut self, world: &'a World) -> Self::Item {
        Commands {
            queue: self,
            entities: &world.entity_generator,
        }
    }
}

impl<'a> SystemParam for Commands<'a> {
    type Fetch = CommandState;
}

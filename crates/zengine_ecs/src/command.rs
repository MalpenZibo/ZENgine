use std::any::TypeId;

use crate::{
    component::ComponentBundle,
    entity::{Entity, EntityGenerator},
    system_parameter::{SystemParam, SystemParamFetch},
    world::{Resource, UnsendableResource, World},
};

pub trait Command: ApplyCommand {
    fn apply(self, world: &mut World);
}

pub trait ApplyCommand {
    fn apply_boxed(self: Box<Self>, world: &mut World);
}

impl<T: Command> ApplyCommand for T {
    fn apply_boxed(self: Box<Self>, world: &mut World) {
        self.apply(world)
    }
}

type CommandState = Vec<Box<dyn Command>>;

struct SpawnCommand<T: ComponentBundle> {
    entity: Entity,
    components: T,
}

impl<T: ComponentBundle> Command for SpawnCommand<T> {
    fn apply(self, world: &mut World) {
        world.spawn_reserved(self.entity, self.components);
    }
}

struct DespawnCommand {
    entity: Entity,
}

impl Command for DespawnCommand {
    fn apply(self, world: &mut World) {
        world.despawn(self.entity);
    }
}

struct CreateResourceCommand<T: Resource> {
    resource: T,
}

impl<T: Resource> Command for CreateResourceCommand<T> {
    fn apply(self, world: &mut World) {
        world.create_resource(self.resource);
    }
}

struct DestroyResourceCommand {
    resource_type: TypeId,
}

impl Command for DestroyResourceCommand {
    fn apply(self, world: &mut World) {
        world.destroy_resource_with_type_id(self.resource_type);
    }
}

struct CreateUnsendableResourceCommand<T: UnsendableResource> {
    resource: T,
}

impl<T: UnsendableResource> Command for CreateUnsendableResourceCommand<T> {
    fn apply(self, world: &mut World) {
        world.create_unsendable_resource(self.resource);
    }
}

struct DestroyUnsendableResourceCommand {
    resource_type: TypeId,
}

impl Command for DestroyUnsendableResourceCommand {
    fn apply(self, world: &mut World) {
        world.destroy_unsendable_resource_with_type_id(self.resource_type);
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

    pub fn create_unsendable_resource<T: UnsendableResource>(&mut self, resource: T) {
        self.queue
            .push(Box::new(CreateUnsendableResourceCommand { resource }))
    }

    pub fn destroy_unsendable_resource<T: UnsendableResource>(&mut self) {
        self.queue.push(Box::new(DestroyUnsendableResourceCommand {
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

    fn apply(&mut self, world: &mut World) {
        for q in self.drain(0..) {
            q.apply_boxed(world);
        }
    }
}

impl<'a> SystemParam for Commands<'a> {
    type Fetch = CommandState;
}

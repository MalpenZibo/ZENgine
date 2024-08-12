use std::{any::TypeId, marker::PhantomData};

use crate::{ComponentBundle, Entity, EntityGenerator, Resource, UnsendableResource, World};

use super::SystemParam;

type CommandState = Vec<Box<dyn Command>>;

pub struct Commands<'a, 'b> {
    queue: &'b mut CommandState,
    entities: &'a EntityGenerator,
}

impl<'a, 'b> SystemParam for Commands<'a, 'b> {
    type State = CommandState;
    type Item<'w, 's> = Commands<'w, 's>;

    fn get<'w, 's>(world: &'w World, state: &'s mut Self::State) -> Self::Item<'w, 's> {
        Commands {
            queue: state,
            entities: &world.entity_generator,
        }
    }

    fn apply(world: &mut World, state: &mut Self::State) {
        for q in state.drain(0..) {
            q.apply_boxed(world);
        }
    }
}

impl<'a, 'b> Commands<'a, 'b> {
    /// Spawn a new entity with the given Components tuple
    pub fn spawn<T: ComponentBundle + 'static>(&mut self, component_bundle: T) -> Entity {
        let entity = self.entities.generate();
        self.queue.push(Box::new(SpawnCommand {
            entity,
            components: component_bundle,
        }));

        entity
    }

    /// Despawn the given [Entity]
    pub fn despawn(&mut self, entity: Entity) {
        self.queue.push(Box::new(DespawnCommand { entity }))
    }

    /// Add the given components tuple to the given [Entity]
    pub fn add_components<T: ComponentBundle + 'static>(
        &mut self,
        entity: Entity,
        component_bundle: T,
    ) {
        self.queue.push(Box::new(AddComponentCommand {
            entity,
            components: component_bundle,
        }))
    }

    /// Removes the given components tuple type from the given [Entity]
    pub fn remove_components<T: ComponentBundle + 'static>(&mut self, entity: Entity) {
        self.queue.push(Box::new(RemoveComponentCommand::<T> {
            entity,
            _phantom: PhantomData,
        }))
    }

    /// Create or replace the given [Resource]
    pub fn create_resource<T: Resource + Send + Sync>(&mut self, resource: T) {
        self.queue
            .push(Box::new(CreateResourceCommand { resource }))
    }

    /// Destroy the given [Resource] type
    pub fn destroy_resource<T: Resource + Send + Sync>(&mut self) {
        self.queue.push(Box::new(DestroyResourceCommand {
            resource_type: TypeId::of::<T>(),
        }))
    }

    /// Create or replace the given [UnsendableResource]
    pub fn create_unsendable_resource<T: UnsendableResource>(&mut self, resource: T) {
        self.queue
            .push(Box::new(CreateUnsendableResourceCommand { resource }))
    }

    /// Destroy the given [UnsendableResource] type
    pub fn destroy_unsendable_resource<T: UnsendableResource>(&mut self) {
        self.queue.push(Box::new(DestroyUnsendableResourceCommand {
            resource_type: TypeId::of::<T>(),
        }))
    }
}

#[doc(hidden)]
pub trait Command: ApplyCommand {
    fn apply(self, world: &mut World);
}

#[doc(hidden)]
pub trait ApplyCommand: Send + Sync {
    fn apply_boxed(self: Box<Self>, world: &mut World);
}

impl<T: Command> ApplyCommand for T {
    fn apply_boxed(self: Box<Self>, world: &mut World) {
        self.apply(world)
    }
}

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

struct AddComponentCommand<T: ComponentBundle> {
    entity: Entity,
    components: T,
}

impl<T: ComponentBundle> Command for AddComponentCommand<T> {
    fn apply(self, world: &mut World) {
        world.add_component(self.entity, self.components);
    }
}

struct RemoveComponentCommand<T: ComponentBundle> {
    entity: Entity,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: ComponentBundle> Command for RemoveComponentCommand<T> {
    fn apply(self, world: &mut World) {
        world.remove_component::<T>(self.entity);
    }
}

struct CreateResourceCommand<T: Resource> {
    resource: T,
}

impl<T: Resource + Sync + Send> Command for CreateResourceCommand<T> {
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
unsafe impl<T: UnsendableResource> Send for CreateUnsendableResourceCommand<T> {}
unsafe impl<T: UnsendableResource> Sync for CreateUnsendableResourceCommand<T> {}

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

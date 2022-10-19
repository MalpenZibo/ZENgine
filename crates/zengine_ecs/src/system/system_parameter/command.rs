use super::{SystemParam, SystemParamFetch};
use crate::{
    component::ComponentBundle,
    entity::{Entity, EntityGenerator},
    Resource, UnsendableResource, World,
};
use std::any::TypeId;

#[doc(hidden)]
pub trait Command: ApplyCommand {
    fn apply(self, world: &mut World);
}

#[doc(hidden)]
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

/// A queue of commands that get executed at the end of the stage of the system that called them
///
/// Each command can be used to modify the World in arbitrary ways:
///
/// - spawning or despawning entities
/// - adding or removing components on existing entities
/// - destroy and create resources
///
/// # Example
/// ```
/// fn my_system(mut commands: Commands) {
///     commands.spawn((ComponentA {}, ComponentB {}));
/// }
/// ```
pub struct Commands<'a> {
    queue: &'a mut CommandState,
    entities: &'a EntityGenerator,
}

impl<'a> Commands<'a> {
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
            _phantom: std::marker::PhantomData::default(),
        }))
    }

    /// Create or replace the given [Resource]
    pub fn create_resource<T: Resource>(&mut self, resource: T) {
        self.queue
            .push(Box::new(CreateResourceCommand { resource }))
    }

    /// Destroy the given [Resource] type
    pub fn destroy_resource<T: Resource>(&mut self) {
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

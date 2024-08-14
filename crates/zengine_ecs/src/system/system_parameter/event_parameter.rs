use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use crate::event::{EventHandler, SubscriptionToken};

use super::{ReadOnlySystemParam, SystemParam};

/// Shared borrow of an event with a subscription to the event queue
///
/// # Example
/// ```
/// use zengine_ecs::system::EventStream;
///
/// #[derive(Debug)]
/// struct EventA {}
///
/// fn my_system(event: EventStream<EventA>) {
///     for e in event.read() {
///         println!("Event {:?}", e);
///     }
/// }
/// ```
pub struct EventStream<'a, E: Send + Sync + std::fmt::Debug + 'static> {
    event_handler: RwLockReadGuard<'a, EventHandler<E>>,
    token: SubscriptionToken,
}

impl<'a, E: Send + Sync + std::fmt::Debug + 'static> EventStream<'a, E> {
    pub fn read(&self) -> impl Iterator<Item = &E> {
        self.event_handler.read(&self.token)
    }
}

impl<'a, E: Send + Sync + std::fmt::Debug + 'static> SystemParam for EventStream<'a, E> {
    type State = Option<SubscriptionToken>;
    type Item<'w, 's> = EventStream<'w, E>;

    fn init(world: &mut crate::World, state: &mut Self::State) {
        if world.get_event_handler::<E>().is_none() {
            world.create_event_handler::<E>()
        }

        *state = world
            .get_mut_event_handler::<E>()
            .map(|mut e: RwLockWriteGuard<EventHandler<E>>| e.subscribe());
    }

    fn get<'w, 's>(world: &'w crate::World, state: &'s mut Self::State) -> Self::Item<'w, 's> {
        Self::Item {
            event_handler: world.get_event_handler().unwrap(),
            token: state.unwrap(),
        }
    }
}

impl<'a, E: Send + Sync + std::fmt::Debug + 'static> ReadOnlySystemParam for EventStream<'a, E> {}

/// Shared borrow of an event without a subscription to the event queue
///
/// # Example
/// ```
/// use zengine_ecs::system::Event;
///
/// #[derive(Debug)]
/// struct EventA {}
///
/// fn my_system(event: Event<EventA>) {
///     if let Some(e) = event.read() {
///         println!("Event {:?}", e);
///     }
/// }
/// ```
pub struct Event<'a, E: Send + Sync + std::fmt::Debug + 'static> {
    event_handler: RwLockReadGuard<'a, EventHandler<E>>,
}

impl<'a, E: Send + Sync + std::fmt::Debug + 'static> Event<'a, E> {
    pub fn read(&self) -> Option<&E> {
        self.event_handler.read_last()
    }
}

impl<'a, E: Send + Sync + std::fmt::Debug + 'static> SystemParam for Event<'a, E> {
    type State = ();
    type Item<'w, 's> = Event<'w, E>;

    fn init(world: &mut crate::World, _state: &mut Self::State) {
        if world.get_event_handler::<E>().is_none() {
            world.create_event_handler::<E>()
        }
    }

    fn get<'w, 's>(world: &'w crate::World, _state: &'s mut Self::State) -> Self::Item<'w, 's> {
        Self::Item {
            event_handler: world.get_event_handler().unwrap(),
        }
    }
}

impl<'a, E: Send + Sync + std::fmt::Debug + 'static> ReadOnlySystemParam for Event<'a, E> {}

/// Unique mutable borrow of an event
///
/// # Example
/// ```
/// use zengine_ecs::system::EventPublisher;
///
/// #[derive(Debug)]
/// struct EventA {}
///
/// fn my_system(mut event: EventPublisher<EventA>) {
///    let new_event = EventA {};
///    event.publish(new_event);
/// }
pub struct EventPublisher<'a, E: Send + Sync + std::fmt::Debug + 'static> {
    event_handler: RwLockWriteGuard<'a, EventHandler<E>>,
}

impl<'a, E: Send + Sync + std::fmt::Debug + 'static> EventPublisher<'a, E> {
    pub fn new(event_handler: RwLockWriteGuard<'a, EventHandler<E>>) -> Self {
        Self { event_handler }
    }

    pub fn publish(&mut self, event: E) {
        self.event_handler.publish(event)
    }
}

impl<'a, E: Send + Sync + std::fmt::Debug + 'static> SystemParam for EventPublisher<'a, E> {
    type State = Option<SubscriptionToken>;
    type Item<'w, 's> = EventPublisher<'w, E>;

    fn init(world: &mut crate::World, _state: &mut Self::State) {
        if world.get_event_handler::<E>().is_none() {
            world.create_event_handler::<E>()
        }
    }

    fn get<'w, 's>(world: &'w crate::World, _state: &'s mut Self::State) -> Self::Item<'w, 's> {
        Self::Item {
            event_handler: world.get_mut_event_handler().unwrap(),
        }
    }
}

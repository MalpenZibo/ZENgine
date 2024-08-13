use super::{ConditionParam, ConditionParamFetch, SystemParam, SystemParamFetch};
use crate::{
    event::{EventHandler, SubscriptionToken},
    world::World,
};
use std::{
    marker::PhantomData,
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

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

#[doc(hidden)]
pub struct EventStreamState<E: Send + Sync + std::fmt::Debug + 'static> {
    _marker: std::marker::PhantomData<E>,
    token: Option<SubscriptionToken>,
}

impl<E: Send + Sync + std::fmt::Debug + 'static> Default for EventStreamState<E> {
    fn default() -> Self {
        EventStreamState {
            _marker: PhantomData,
            token: None,
        }
    }
}

impl<'a, E: Send + Sync + std::fmt::Debug + 'static> SystemParamFetch<'a> for EventStreamState<E> {
    type Item = EventStream<'a, E>;

    fn init(&mut self, world: &mut World) {
        if world.get_event_handler::<E>().is_none() {
            world.create_event_handler::<E>()
        }

        self.token = world
            .get_mut_event_handler::<E>()
            .map(|mut e: RwLockWriteGuard<EventHandler<E>>| e.subscribe());
    }

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        Self::Item {
            event_handler: world.get_event_handler().unwrap(),
            token: self.token.unwrap(),
        }
    }
}

impl<'a, E: Send + Sync + std::fmt::Debug + 'static> SystemParam for EventStream<'a, E> {
    type Fetch = EventStreamState<E>;
}

impl<'a, E: Send + Sync + std::fmt::Debug + 'static> ConditionParamFetch<'a>
    for EventStreamState<E>
{
    type Item = EventStream<'a, E>;

    fn init(&mut self, world: &mut World) {
        if world.get_event_handler::<E>().is_none() {
            world.create_event_handler::<E>()
        }

        self.token = world
            .get_mut_event_handler::<E>()
            .map(|mut e: RwLockWriteGuard<EventHandler<E>>| e.subscribe());
    }

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        Self::Item {
            event_handler: world.get_event_handler().unwrap(),
            token: self.token.unwrap(),
        }
    }
}

impl<'a, E: Send + Sync + std::fmt::Debug + 'static> ConditionParam for EventStream<'a, E> {
    type Fetch = EventStreamState<E>;
}

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

#[doc(hidden)]
pub struct EventState<E: Send + Sync + std::fmt::Debug + 'static> {
    _marker: std::marker::PhantomData<E>,
}

impl<E: Send + Sync + std::fmt::Debug + 'static> Default for EventState<E> {
    fn default() -> Self {
        EventState {
            _marker: PhantomData,
        }
    }
}

impl<'a, E: Send + Sync + std::fmt::Debug + 'static> SystemParamFetch<'a> for EventState<E> {
    type Item = Event<'a, E>;

    fn init(&mut self, world: &mut World) {
        if world.get_event_handler::<E>().is_none() {
            world.create_event_handler::<E>()
        }
    }

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        Self::Item {
            event_handler: world.get_event_handler().unwrap(),
        }
    }
}

impl<'a, E: Send + Sync + std::fmt::Debug + 'static> SystemParam for Event<'a, E> {
    type Fetch = EventState<E>;
}

impl<'a, E: Send + Sync + std::fmt::Debug + 'static> ConditionParamFetch<'a> for EventState<E> {
    type Item = Event<'a, E>;

    fn init(&mut self, world: &mut World) {
        if world.get_event_handler::<E>().is_none() {
            world.create_event_handler::<E>()
        }
    }

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        Self::Item {
            event_handler: world.get_event_handler().unwrap(),
        }
    }
}

impl<'a, E: Send + Sync + std::fmt::Debug + 'static> ConditionParam for Event<'a, E> {
    type Fetch = EventState<E>;
}

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

#[doc(hidden)]
pub struct EventPublisherState<E: Send + Sync + std::fmt::Debug + 'static> {
    _marker: std::marker::PhantomData<E>,
}

impl<E: Send + Sync + std::fmt::Debug + 'static> Default for EventPublisherState<E> {
    fn default() -> Self {
        EventPublisherState {
            _marker: PhantomData,
        }
    }
}

impl<'a, E: Send + Sync + std::fmt::Debug + 'static> SystemParamFetch<'a>
    for EventPublisherState<E>
{
    type Item = EventPublisher<'a, E>;

    fn init(&mut self, world: &mut World) {
        if world.get_event_handler::<E>().is_none() {
            world.create_event_handler::<E>()
        }
    }

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        Self::Item {
            event_handler: world.get_mut_event_handler().unwrap(),
        }
    }
}

impl<'a, E: Send + Sync + std::fmt::Debug + 'static> SystemParam for EventPublisher<'a, E> {
    type Fetch = EventPublisherState<E>;
}

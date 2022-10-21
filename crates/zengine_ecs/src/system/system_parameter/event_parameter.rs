use super::{SystemParam, SystemParamFetch};
use crate::{
    event::{EventHandler, SubscriptionToken},
    world::World,
};
use std::{
    any::Any,
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
pub struct EventStream<'a, E: Any + std::fmt::Debug> {
    event_handler: RwLockReadGuard<'a, EventHandler<E>>,
    token: SubscriptionToken,
}

impl<'a, E: Any + std::fmt::Debug> EventStream<'a, E> {
    pub fn read(&self) -> impl Iterator<Item = &E> {
        self.event_handler.read(&self.token)
    }
}

#[doc(hidden)]
pub struct EventStreamState<E: Any + std::fmt::Debug> {
    _marker: std::marker::PhantomData<E>,
    token: Option<SubscriptionToken>,
}

impl<E: Any + std::fmt::Debug> Default for EventStreamState<E> {
    fn default() -> Self {
        EventStreamState {
            _marker: std::marker::PhantomData::default(),
            token: None,
        }
    }
}

impl<'a, E: Any + std::fmt::Debug> SystemParamFetch<'a> for EventStreamState<E> {
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

impl<'a, E: Any + std::fmt::Debug> SystemParam for EventStream<'a, E> {
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
pub struct Event<'a, E: Any + std::fmt::Debug> {
    event_handler: RwLockReadGuard<'a, EventHandler<E>>,
}

impl<'a, E: Any + std::fmt::Debug> Event<'a, E> {
    pub fn read(&self) -> Option<&E> {
        self.event_handler.read_last()
    }
}

#[doc(hidden)]
pub struct EventState<E: Any + std::fmt::Debug> {
    _marker: std::marker::PhantomData<E>,
}

impl<E: Any + std::fmt::Debug> Default for EventState<E> {
    fn default() -> Self {
        EventState {
            _marker: std::marker::PhantomData::default(),
        }
    }
}

impl<'a, E: Any + std::fmt::Debug> SystemParamFetch<'a> for EventState<E> {
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

impl<'a, E: Any + std::fmt::Debug> SystemParam for Event<'a, E> {
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
pub struct EventPublisher<'a, E: Any + std::fmt::Debug> {
    event_handler: RwLockWriteGuard<'a, EventHandler<E>>,
}

impl<'a, E: Any + std::fmt::Debug> EventPublisher<'a, E> {
    pub fn new(event_handler: RwLockWriteGuard<'a, EventHandler<E>>) -> Self {
        Self { event_handler }
    }

    pub fn publish(&mut self, event: E) {
        self.event_handler.publish(event)
    }
}

#[doc(hidden)]
pub struct EventPublisherState<E: Any + std::fmt::Debug> {
    _marker: std::marker::PhantomData<E>,
}

impl<E: Any + std::fmt::Debug> Default for EventPublisherState<E> {
    fn default() -> Self {
        EventPublisherState {
            _marker: std::marker::PhantomData::default(),
        }
    }
}

impl<'a, E: Any + std::fmt::Debug> SystemParamFetch<'a> for EventPublisherState<E> {
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

impl<'a, E: Any + std::fmt::Debug> SystemParam for EventPublisher<'a, E> {
    type Fetch = EventPublisherState<E>;
}

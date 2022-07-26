use std::{
    any::Any,
    cell::{Ref, RefMut},
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use crate::{
    event::{EventHandler, SubscriptionToken},
    query::{Query, QueryCache, QueryParameters},
    world::{Resource, UnsendableResource, World},
};

pub trait SystemParam: Sized {
    type Fetch: for<'a> SystemParamFetch<'a> + Default;
}

pub trait SystemParamFetch<'a> {
    type Item;

    fn init(&mut self, _world: &mut World) {}

    fn fetch(&'a mut self, world: &'a World) -> Self::Item;

    fn apply(&mut self, _world: &mut World) {}
}

pub type SystemParamItem<'a, P> = <<P as SystemParam>::Fetch as SystemParamFetch<'a>>::Item;

pub struct QueryState<T: QueryParameters> {
    _marker: std::marker::PhantomData<T>,
    query_cache: Option<QueryCache>,
}

impl<T: QueryParameters> Default for QueryState<T> {
    fn default() -> Self {
        QueryState {
            _marker: std::marker::PhantomData::default(),
            query_cache: None,
        }
    }
}

impl<'a, T: QueryParameters> SystemParamFetch<'a> for QueryState<T> {
    type Item = Query<'a, T>;

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        Query {
            data: T::fetch(world, &mut self.query_cache),
        }
    }
}

impl<'a, T: QueryParameters> SystemParam for Query<'a, T> {
    type Fetch = QueryState<T>;
}

pub type Res<'a, R> = RwLockReadGuard<'a, R>;

pub struct ResState<R: Resource + Default> {
    _marker: std::marker::PhantomData<R>,
}

impl<T: Resource + Default> Default for ResState<T> {
    fn default() -> Self {
        ResState {
            _marker: std::marker::PhantomData::default(),
        }
    }
}

impl<'a, R: Resource + Default> SystemParamFetch<'a> for ResState<R> {
    type Item = Res<'a, R>;

    fn init(&mut self, world: &mut World) {
        if world.get_resource::<R>().is_none() {
            world.create_resource::<R>(R::default())
        }
    }

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        world.get_resource().unwrap()
    }
}

impl<'a, R: Resource + Default> SystemParam for Res<'a, R> {
    type Fetch = ResState<R>;
}

pub type ResMut<'a, R> = RwLockWriteGuard<'a, R>;

pub struct ResMutState<R: Resource + Default> {
    _marker: std::marker::PhantomData<R>,
}

impl<T: Resource + Default> Default for ResMutState<T> {
    fn default() -> Self {
        ResMutState {
            _marker: std::marker::PhantomData::default(),
        }
    }
}

impl<'a, R: Resource + Default> SystemParamFetch<'a> for ResMutState<R> {
    type Item = ResMut<'a, R>;

    fn init(&mut self, world: &mut World) {
        if world.get_resource::<R>().is_none() {
            world.create_resource::<R>(R::default())
        }
    }

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        world.get_mut_resource().unwrap()
    }
}

impl<'a, R: Resource + Default> SystemParam for ResMut<'a, R> {
    type Fetch = ResMutState<R>;
}

pub type OptionalRes<'a, R> = Option<RwLockReadGuard<'a, R>>;

pub struct OptionalResState<R: Resource> {
    _marker: std::marker::PhantomData<R>,
}

impl<T: Resource> Default for OptionalResState<T> {
    fn default() -> Self {
        OptionalResState {
            _marker: std::marker::PhantomData::default(),
        }
    }
}

impl<'a, R: Resource> SystemParamFetch<'a> for OptionalResState<R> {
    type Item = OptionalRes<'a, R>;

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        world.get_resource()
    }
}

impl<'a, R: Resource> SystemParam for OptionalRes<'a, R> {
    type Fetch = OptionalResState<R>;
}

pub type OptionalUnsendableRes<'a, R> = Option<Ref<'a, R>>;

pub struct OptionalUnsendableResState<R: UnsendableResource> {
    _marker: std::marker::PhantomData<R>,
}

impl<T: UnsendableResource> Default for OptionalUnsendableResState<T> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData::default(),
        }
    }
}

impl<'a, R: UnsendableResource> SystemParamFetch<'a> for OptionalUnsendableResState<R> {
    type Item = OptionalUnsendableRes<'a, R>;

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        world.get_unsendable_resource()
    }
}

impl<'a, R: UnsendableResource> SystemParam for OptionalUnsendableRes<'a, R> {
    type Fetch = OptionalUnsendableResState<R>;
}

pub type OptionalUnsendableResMut<'a, R> = Option<RefMut<'a, R>>;

pub struct OptionalUnsendableResMutState<R: UnsendableResource> {
    _marker: std::marker::PhantomData<R>,
}

impl<T: UnsendableResource> Default for OptionalUnsendableResMutState<T> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData::default(),
        }
    }
}

impl<'a, R: UnsendableResource> SystemParamFetch<'a> for OptionalUnsendableResMutState<R> {
    type Item = OptionalUnsendableResMut<'a, R>;

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        world.get_mut_unsendable_resource()
    }
}

impl<'a, R: UnsendableResource> SystemParam for OptionalUnsendableResMut<'a, R> {
    type Fetch = OptionalUnsendableResMutState<R>;
}

pub type OptionalResMut<'a, R> = Option<RwLockWriteGuard<'a, R>>;

pub struct OptionalResMutState<R: Resource> {
    _marker: std::marker::PhantomData<R>,
}

impl<T: Resource> Default for OptionalResMutState<T> {
    fn default() -> Self {
        OptionalResMutState {
            _marker: std::marker::PhantomData::default(),
        }
    }
}

impl<'a, R: Resource> SystemParamFetch<'a> for OptionalResMutState<R> {
    type Item = OptionalResMut<'a, R>;

    fn fetch(&mut self, world: &'a World) -> Self::Item {
        world.get_mut_resource()
    }
}

impl<'a, R: Resource> SystemParam for OptionalResMut<'a, R> {
    type Fetch = OptionalResMutState<R>;
}

pub struct EventStream<'a, E: Any + std::fmt::Debug> {
    event_handler: RwLockReadGuard<'a, EventHandler<E>>,
    token: SubscriptionToken,
}

impl<'a, E: Any + std::fmt::Debug> EventStream<'a, E> {
    pub fn read(&self) -> impl Iterator<Item = &E> {
        self.event_handler.read(&self.token)
    }
}

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

pub struct Event<'a, E: Any + std::fmt::Debug> {
    event_handler: RwLockReadGuard<'a, EventHandler<E>>,
}

impl<'a, E: Any + std::fmt::Debug> Event<'a, E> {
    pub fn read(&self) -> Option<&E> {
        self.event_handler.read_last()
    }
}

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

pub struct EventPublisher<'a, E: Any + std::fmt::Debug> {
    event_handler: RwLockWriteGuard<'a, EventHandler<E>>,
}

impl<'a, E: Any + std::fmt::Debug> EventPublisher<'a, E> {
    pub fn publish(&mut self, event: E) {
        self.event_handler.publish(event)
    }
}

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

pub type Local<'a, T> = &'a mut T;

pub struct LocalState<T: Default> {
    data: T,
}

impl<T: Default + 'static> Default for LocalState<T> {
    fn default() -> Self {
        LocalState { data: T::default() }
    }
}

impl<'a, T: Default + 'static> SystemParamFetch<'a> for LocalState<T> {
    type Item = Local<'a, T>;

    fn fetch(&'a mut self, _world: &'a World) -> Self::Item {
        &mut self.data
    }
}

impl<'a, T: Default + 'static> SystemParam for Local<'a, T> {
    type Fetch = LocalState<T>;
}

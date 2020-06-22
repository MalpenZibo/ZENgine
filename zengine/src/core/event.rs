use crate::core::store::Resource;
use fnv::FnvHashMap;
use std::any::Any;
use std::any::TypeId;
use std::cell::RefCell;
use std::cell::RefMut;
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug)]
pub struct EventStream<E: Any> {
    buffer: Vec<E>,
    subscriptions: FnvHashMap<SubscriptionToken, RefCell<Subscription>>,
    token_serial: u64,
}

impl<E: Any> Default for EventStream<E> {
    fn default() -> Self {
        EventStream {
            buffer: Vec::default(),
            subscriptions: FnvHashMap::default(),
            token_serial: 0,
        }
    }
}

impl<E: Any> Resource for EventStream<E> {}

impl<E: Any> EventStream<E> {
    pub fn subscribe(&mut self) -> SubscriptionToken {
        let token = self.generate_token();
        self.subscriptions
            .insert(token.clone(), RefCell::new(Subscription { position: 0 }));

        token
    }

    pub fn unsubscribe(&mut self, token: SubscriptionToken) {
        self.subscriptions.remove(&token);
    }

    pub fn publish(&mut self, event: E) {
        self.buffer.push(event);
    }

    pub fn read_last(&self) -> Option<&E> {
        self.buffer.last()
    }

    pub fn read(&self, token: &SubscriptionToken) -> &[E] {
        let mut subscription = self.get_subscription(token);
        let start_index = subscription.position;
        subscription.position = self.buffer.len();

        &self.buffer[start_index..]
    }

    fn get_subscription(&self, token: &SubscriptionToken) -> RefMut<Subscription> {
        self.subscriptions
            .get(token)
            .expect("Invalid token suplied")
            .borrow_mut()
    }

    fn generate_token(&mut self) -> SubscriptionToken {
        let new_token = self.token_serial;
        self.token_serial += 1;

        SubscriptionToken(new_token)
    }
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct SubscriptionKey {
    pub event_id: TypeId,
    pub token: SubscriptionToken,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct SubscriptionToken(u64);

#[derive(Debug)]
pub struct Subscription {
    pub position: usize,
}
/*
pub struct StreamReader<'a, E: Event> {
    stream: Ref<'a, EventStream<E>>,
    subscription: RefMut<'a, Subscription>,
}

impl<'a, E: Event> StreamReader<'a, E> {
    pub fn read(&self) -> &[E] {
        self.stream.read(&mut self.subscription)
    }
}

#[derive(Debug, Default)]
pub struct EventBus {
    streams: FnvHashMap<TypeId, RefCell<Box<dyn AnyStream>>>,
    subscribers: FnvHashMap<SubscriptionKey, RefCell<Subscription>>,
    subscriber_id: u64,
}

impl Resource for EventBus {}

impl EventBus {
    pub fn register_stream<E: Event>(&mut self) {
        let event_id = TypeId::of::<E>();

        self.streams.insert(
            event_id,
            RefCell::new(Box::new(EventStream::<E>::default())),
        );
    }

    pub fn get_reader<E: Event>(&self, token: SubscriptionToken) -> Option<StreamReader<E>> {
        let event_id = TypeId::of::<E>();

        let stream = match self.streams.get(&event_id) {
            Some(stream) => Some(Ref::map(stream.borrow(), |b| {
                b.downcast_ref::<EventStream<E>>()
                    .expect("downcast set error")
            })),
            None => None,
        };

        let subscription = match self.subscribers.get_mut(&SubscriptionKey {
            event_id: event_id,
            token: token,
        }) {
            Some(subscription) => Some(subscription.borrow_mut()),
            None => None,
        };

        match (stream, subscription) {
            (Some(stream), Some(subscription)) => Some(StreamReader {
                stream: stream,
                subscription: subscription,
            }),
            _ => None,
        }
    }

    pub fn get_publisher<E: Event>(&self) -> Option<RefMut<EventStream<E>>> {
        let type_id = TypeId::of::<E>();

        match self.streams.get_mut(&type_id) {
            Some(stream) => Some(RefMut::map(stream.borrow_mut(), |b| {
                b.downcast_mut::<EventStream<E>>()
                    .expect("downcast set error")
            })),
            None => None,
        }
    }

    pub fn subscribe<E: Event>(mut self) -> SubscriptionToken {
        let event_id = TypeId::of::<E>();

        let stream = self.streams.get(&event_id).or_else(|| {
            self.streams.insert(
                event_id,
                RefCell::new(Box::new(EventStream::<E>::default())),
            );
            self.streams.get(&event_id)
        });

        let key = self.generate_key::<E>();
        let token = key.token;

        self.subscribers
            .insert(key, RefCell::new(Subscription { position: 0 }));

        token
    }

    fn generate_key<E: Event>(&mut self) -> SubscriptionKey {
        let event_id = TypeId::of::<E>();
        let token = self.subscriber_id;
        self.subscriber_id += 1;

        SubscriptionKey {
            event_id: event_id,
            token: SubscriptionToken(token),
        }
    }
}

#[derive(Copy, Clone)]
pub enum SubscriptionState {
    Enabled,
    Disabled,
}

pub trait AnyReader: Downcast {
    fn get_position(&self) -> usize;

    fn set_position(&mut self, new_position: usize);
}
downcast_rs::impl_downcast!(AnyReader);
*/
/*
impl<'a, E: Any, T: Any> AnyReader for Reader<E, T> {
    fn get_position(&self) -> usize {
        self.position
    }

    fn set_position(&mut self, new_position: usize) {
        self.position = new_position;
    }
}
*/
//impl<E: Any, T: Any> Resource for Reader<E, T> {}

//impl<E: Any> Resource for EventStream<E> {}

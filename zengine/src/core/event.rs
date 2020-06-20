use crate::core::store::Resource;
use downcast_rs::Downcast;
use std::any::Any;
use std::marker::PhantomData;

#[derive(Copy, Clone)]
pub enum SubscriptionState {
    Enabled,
    Disabled,
}

pub struct Reader<E, R> {
    position: usize,
    phantom_event: PhantomData<E>,
    phantom_system: PhantomData<R>,
}

impl<E, R> Reader<E, R> {
    pub fn init() -> Self {
        Reader {
            position: 0,
            phantom_event: PhantomData::default(),
            phantom_system: PhantomData::default(),
        }
    }
}

pub trait AnyReader: Downcast {
    fn get_position(&self) -> usize;

    fn set_position(&mut self, new_position: usize);
}
downcast_rs::impl_downcast!(AnyReader);

impl<'a, E: Any, T: Any> AnyReader for Reader<E, T> {
    fn get_position(&self) -> usize {
        self.position
    }

    fn set_position(&mut self, new_position: usize) {
        self.position = new_position;
    }
}

impl<E: Any, T: Any> Resource for Reader<E, T> {}

pub struct EventStream<E> {
    buffer: Vec<E>,
}

impl<E> EventStream<E> {
    pub fn init() -> Self {
        EventStream {
            buffer: Vec::default(),
        }
    }

    pub fn publish(&mut self, event: E) {
        self.buffer.push(event)
    }

    pub fn read(&self, reader_position: usize) -> (&[E], usize) {
        let slice = &self.buffer[reader_position..];
        (slice, self.buffer.len())
    }

    pub fn read_last(&self) -> Option<&E> {
        self.buffer.last()
    }
}

impl<E: Any> Resource for EventStream<E> {}

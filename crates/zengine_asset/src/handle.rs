use std::{any::TypeId, marker::PhantomData};

use crossbeam_channel::{Receiver, Sender};

use crate::assets::{Asset, Assets};

pub type HandleId = (TypeId, u64);

pub(crate) enum HandleEvent {
    Increment(HandleId),
    Decrement(HandleId),
}

pub(crate) enum HandleType {
    Strong(Sender<HandleEvent>),
    Weak,
}

pub struct Handle<T> {
    pub id: HandleId,
    handle_type: HandleType,
    _phantom: PhantomData<T>,
}

impl<T> Drop for Handle<T> {
    fn drop(&mut self) {
        if let HandleType::Strong(sender) = &self.handle_type {
            sender.send(HandleEvent::Decrement(self.id)).unwrap();
        }
    }
}

impl<T: Asset> Clone for Handle<T> {
    fn clone(&self) -> Self {
        match self.handle_type {
            HandleType::Strong(ref sender) => Handle::strong(self.id, sender.clone()),
            HandleType::Weak => Handle::weak(self.id),
        }
    }
}

impl<T: Asset> Handle<T> {
    pub(crate) fn strong(id: HandleId, handle_event_sender: Sender<HandleEvent>) -> Self {
        handle_event_sender
            .send(HandleEvent::Increment(id))
            .unwrap();
        Self {
            id,
            handle_type: HandleType::Strong(handle_event_sender),
            _phantom: PhantomData::default(),
        }
    }

    pub fn weak(id: HandleId) -> Self {
        Self {
            id,
            handle_type: HandleType::Weak,
            _phantom: PhantomData::default(),
        }
    }

    pub fn as_weak(&self) -> Self {
        Handle::weak(self.id)
    }

    pub fn is_strong(&self) -> bool {
        matches!(self.handle_type, HandleType::Strong(_))
    }

    pub fn is_weak(&self) -> bool {
        matches!(self.handle_type, HandleType::Weak)
    }

    pub fn make_strong(&mut self, assets: &Assets<T>) {
        if self.is_weak() {
            let sender = assets.sender.clone();
            sender.send(HandleEvent::Increment(self.id)).unwrap();
            self.handle_type = HandleType::Strong(sender)
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct HandleEventChannel {
    pub sender: Sender<HandleEvent>,
    pub receiver: Receiver<HandleEvent>,
}

impl Default for HandleEventChannel {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self { sender, receiver }
    }
}

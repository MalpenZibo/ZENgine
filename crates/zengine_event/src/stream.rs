use fnv::FnvHashMap;
use zengine_ecs::world::Resource;
use std::any::{Any, TypeId};
use std::cmp::Ordering;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{RwLock, RwLockWriteGuard};

const STREAM_SIZE_BLOCK: usize = 10;

#[derive(Debug)]
pub struct EventStream<E: Any> {
    buffer: Vec<E>,
    head: Option<usize>,
    subscriptions: FnvHashMap<SubscriptionToken, RwLock<Subscription>>,
    token_serial: u64,
}

impl<E: Any> Default for EventStream<E> {
    fn default() -> Self {
        EventStream {
            buffer: Vec::with_capacity(STREAM_SIZE_BLOCK),
            head: None,
            subscriptions: FnvHashMap::default(),
            token_serial: 0,
        }
    }
}

impl<E: Any + Send + Sync + Debug> Resource for EventStream<E> {}

impl<E: Any> EventStream<E> {
    pub fn subscribe(&mut self) -> SubscriptionToken {
        let token = self.generate_token();
        self.subscriptions.insert(
            token,
            RwLock::new(Subscription {
                position: self.head,
            }),
        );

        token
    }

    pub fn unsubscribe(&mut self, token: SubscriptionToken) {
        self.subscriptions.remove(&token);
    }

    pub fn publish(&mut self, event: E) {
        if let Some(mut head) = self.head {
            head += 1;
            if head == self.buffer.capacity() {
                head = 0;
            }
            let tail = self.tail();
            if let Some(tail) = tail {
                if head == tail && self.buffer.capacity() == self.buffer.len() {
                    head = self.increase_capacity(tail);
                }
            }
            if self.buffer.len() == self.buffer.capacity() {
                self.buffer[head] = event;
            } else {
                self.buffer.push(event);
            }

            self.head = Some(head);
        } else {
            self.buffer.push(event);
            self.head = Some(0)
        }
    }

    pub fn read_last(&self) -> Option<&E> {
        match self.head {
            Some(head) => self.buffer.get(head),
            _ => None,
        }
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn read(&self, token: &SubscriptionToken) -> impl Iterator<Item = &E> {
        let head = self.head.unwrap_or(0);
        let mut subscription = self.get_subscription(token);
        if self.head != subscription.position {
            let start = match subscription.position {
                Some(pos) if pos + 1 >= self.buffer.len() => 0,
                Some(pos) => pos + 1,
                None => 0,
            };
            subscription.position = self.head;
            self.buffer
                .iter()
                .cycle()
                .skip(start)
                .take(if start >= self.buffer.len() {
                    0
                } else if head >= start {
                    head - start + 1
                } else {
                    self.buffer.len() - (start - head) + 1
                })
        } else {
            self.buffer.iter().cycle().skip(0).take(0)
        }
    }

    fn get_subscription(&self, token: &SubscriptionToken) -> RwLockWriteGuard<Subscription> {
        self.subscriptions
            .get(token)
            .expect("Invalid token suplied")
            .write().unwrap()
    }

    fn generate_token(&mut self) -> SubscriptionToken {
        let new_token = self.token_serial;
        self.token_serial += 1;

        SubscriptionToken(new_token)
    }

    fn tail(&self) -> Option<usize> {
        self.subscriptions
            .values().map(|v| v.read().unwrap())
            .min_by(|a, b| a.cmp(b))
            .map(|sub| sub.position.unwrap_or(0))
    }

    fn increase_capacity(&mut self, tail: usize) -> usize {
        let mut new_buffer: Vec<E> = Vec::with_capacity(self.buffer.capacity() + STREAM_SIZE_BLOCK);

        for e in self.buffer.drain(tail..) {
            new_buffer.push(e);
        }

        for e in self.buffer.drain(0..tail) {
            new_buffer.push(e);
        }

        for s in self.subscriptions.values_mut() {
            let mut sub = s.write().unwrap();
            if let Some(pos) = sub.position {
                if sub.position > self.head {
                    sub.position = Some(pos - tail);
                } else {
                    sub.position = Some(pos + tail);
                }
            }
        }

        let new_head = new_buffer.len();
        self.head = Some(new_head);
        self.buffer = new_buffer;

        new_head
    }
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct SubscriptionKey {
    pub event_id: TypeId,
    pub token: SubscriptionToken,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct SubscriptionToken(u64);

#[derive(Debug, Eq, PartialEq)]
pub struct Subscription {
    pub position: Option<usize>,
}

impl Ord for Subscription {
    fn cmp(&self, other: &Self) -> Ordering {
        self.position.cmp(&other.position)
    }
}

impl PartialOrd for Subscription {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn publish(stream: &mut EventStream<u32>, events: &[u32]) {
        for e in events {
            stream.publish(e.clone());
        }
    }

    #[test]
    fn produce() {
        let mut stream = EventStream::<u32>::default();

        publish(&mut stream, &[4; 4]);

        assert_eq!(stream.head, Some(3));
        assert_eq!(stream.buffer.len(), 4);
    }

    #[test]
    fn produce_and_read() {
        let mut stream = EventStream::<u32>::default();

        let token1 = stream.subscribe();

        publish(&mut stream, &[4; 4]);

        assert_eq!(stream.read(&token1).count(), 4);
    }

    #[test]
    fn produce_and_read_no_new_content() {
        let mut stream = EventStream::<u32>::default();

        let token1 = stream.subscribe();

        publish(&mut stream, &[4; 4]);

        assert_eq!(stream.read(&token1).count(), 4);
        assert_eq!(stream.read(&token1).count(), 0);
    }

    #[test]
    fn produce_buffer_full_and_read_no_new_content() {
        let mut stream = EventStream::<u32>::default();

        let token1 = stream.subscribe();

        publish(&mut stream, &[4; 4]);

        assert_eq!(stream.read(&token1).count(), 4);
        publish(&mut stream, &[4; 6]);
        assert_eq!(stream.read(&token1).count(), 6);

        assert_eq!(stream.head, Some(9));
        assert_eq!(
            stream.subscriptions.get(&token1).unwrap().read().unwrap().position,
            Some(9)
        );
        assert_eq!(stream.buffer.len(), STREAM_SIZE_BLOCK);
        assert_eq!(stream.buffer.capacity(), STREAM_SIZE_BLOCK);

        publish(&mut stream, &[4; 1]);
        assert_eq!(stream.head, Some(0));
        assert_eq!(
            stream.subscriptions.get(&token1).unwrap().read().unwrap().position,
            Some(9)
        );
        assert_eq!(stream.buffer.len(), STREAM_SIZE_BLOCK);
        assert_eq!(stream.buffer.capacity(), STREAM_SIZE_BLOCK);
        assert_eq!(stream.read(&token1).count(), 1);

        assert_eq!(stream.head, Some(0));
        assert_eq!(
            stream.subscriptions.get(&token1).unwrap().read().unwrap().position,
            Some(0)
        );
        assert_eq!(stream.buffer.len(), STREAM_SIZE_BLOCK);
        assert_eq!(stream.buffer.capacity(), STREAM_SIZE_BLOCK);

        assert_eq!(stream.read(&token1).count(), 0);
    }

    #[test]
    fn produce_cycling() {
        let mut stream = EventStream::<u32>::default();

        publish(&mut stream, &[4; 15]);

        assert_eq!(stream.head, Some(4));
        assert_eq!(stream.buffer.len(), STREAM_SIZE_BLOCK);
        assert_eq!(stream.buffer.capacity(), STREAM_SIZE_BLOCK);
    }

    #[test]
    fn buffer_increase() {
        let mut stream = EventStream::<u32>::default();

        stream.subscribe();
        publish(&mut stream, &[4; 15]);

        assert_eq!(stream.head, Some(14));
        assert_eq!(stream.buffer.len(), 15);
        assert_eq!(stream.buffer.capacity(), STREAM_SIZE_BLOCK * 2);
    }

    #[allow(unused_must_use)]
    #[test]
    fn stream_with_lazy_subscriber() {
        let mut stream = EventStream::<u32>::default();

        let token1 = stream.subscribe();
        let token2 = stream.subscribe();
        let token3 = stream.subscribe();

        publish(&mut stream, &[4; 4]);
        stream.read(&token1);
        publish(&mut stream, &[4; 10]);
        stream.read(&token1);
        stream.read(&token2);
        publish(&mut stream, &[4; 50]);
        stream.read(&token1);
        publish(&mut stream, &[4; 3]);
        stream.read(&token1);
        stream.read(&token2);
        stream.read(&token3);

        assert_eq!(stream.buffer.capacity(), STREAM_SIZE_BLOCK * 7);
        assert_eq!(stream.buffer.len(), 67);
        assert_eq!(stream.head, Some(66));
    }

    #[test]
    fn publish_subscribe_and_read() {
        let mut stream = EventStream::<u32>::default();

        publish(&mut stream, &[1]);
        let token1 = stream.subscribe();
        let mut result: Vec<u32> = Vec::new();
        for u in stream.read(&token1) {
            result.push(u.clone())
        }
        assert_eq!(stream.head, Some(0));
        assert_eq!(result, Vec::<u32>::new());
    }

    #[test]
    fn publish_and_read_one_element_after_subscribe() {
        let mut stream = EventStream::<u32>::default();

        let token1 = stream.subscribe();

        publish(&mut stream, &[1]);
        let mut result: Vec<u32> = Vec::new();
        for u in stream.read(&token1) {
            result.push(u.clone())
        }
        assert_eq!(stream.head, Some(0));
        assert_eq!(result, [1]);
    }

    #[test]
    fn sequence_correctness() {
        let mut stream = EventStream::<u32>::default();

        let token1 = stream.subscribe();

        publish(&mut stream, &[1, 2, 3, 4]);
        assert_eq!(stream.tail(), Some(0));
        let mut result: Vec<u32> = Vec::new();
        for u in stream.read(&token1) {
            result.push(u.clone())
        }
        assert_eq!(stream.head, Some(3));
        assert_eq!(result, [1, 2, 3, 4]);

        publish(&mut stream, &[5, 6, 7, 8, 9, 10, 11]);
        let mut result: Vec<u32> = Vec::new();
        assert_eq!(stream.tail(), Some(3));
        for u in stream.read(&token1) {
            result.push(u.clone())
        }
        assert_eq!(stream.head, Some(0));
        assert_eq!(result, [5, 6, 7, 8, 9, 10, 11]);

        publish(
            &mut stream,
            &[
                12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30,
            ],
        );

        assert_eq!(stream.tail(), Some(0));
        let mut result: Vec<u32> = Vec::new();
        for u in stream.read(&token1) {
            result.push(u.clone())
        }
        assert_eq!(stream.head, Some(19));
        assert_eq!(stream.tail(), Some(19));
        assert_eq!(
            result,
            [12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30]
        );
    }
}

use crate::core::component::Component;
use crate::core::component::Set;
use crate::core::entity::Entities;
use crate::core::event::AnyReader;
use crate::core::event::EventStream;
use crate::core::event::Reader;
use crate::core::event::SubscriptionState;
use crate::core::store::Resource;
use crate::core::store::Store;
use log::trace;
use std::any::Any;
use std::any::TypeId;
use std::cell::{Ref, RefMut};

pub trait AnySystem {
    #[allow(unused_variables)]
    fn init(&mut self, store: &mut Store) {}

    fn run_now(&mut self, store: &Store);

    #[allow(unused_variables)]
    fn dispose(&mut self, store: &mut Store) {}
}

impl<S> AnySystem for S
where
    S: for<'a> System<'a>,
{
    fn init(&mut self, store: &mut Store) {
        S::Data::setup::<S>(store);
        self.init(store);
    }

    fn run_now(&mut self, store: &Store) {
        let data = S::Data::fetch::<S>(store);
        self.run(data);
    }

    fn dispose(&mut self, store: &mut Store) {
        self.dispose(store);
    }
}

pub trait System<'a>: Any + Default {
    type Data: Data<'a>;

    #[allow(unused_variables)]
    fn init(&mut self, store: &mut Store) {}

    fn run(&mut self, data: Self::Data);

    #[allow(unused_variables)]
    fn dispose(&mut self, store: &mut Store) {}
}

pub trait Data<'a> {
    fn setup<Sys: System<'a>>(store: &mut Store);

    fn fetch<Sys: System<'a>>(store: &'a Store) -> Self;
}

impl<'a> Data<'a> for () {
    fn setup<Sys: System<'a>>(store: &mut Store) {}

    #[allow(unused_variables)]
    fn fetch<Sys: System<'a>>(store: &'a Store) -> Self {
        ()
    }
}

pub struct ReadStream<'a, E: Any> {
    stream: Ref<'a, EventStream<E>>,
    reader: RefMut<'a, dyn AnyReader>,
}

impl<'a, E: Any> ReadStream<'a, E> {
    pub fn read(&mut self) -> &[E] {
        let read_data = self.stream.read(self.reader.get_position());
        self.reader.set_position(read_data.1);
        read_data.0
    }
}

pub struct WriteStream<'a, E: Any> {
    stream: RefMut<'a, EventStream<E>>,
    reader: RefMut<'a, dyn AnyReader>,
}

impl<'a, E: Any> WriteStream<'a, E> {
    pub fn publish(&mut self, event: E) {
        self.stream.publish(event);
    }
    pub fn read(&mut self) -> &[E] {
        let read_data = self.stream.read(self.reader.get_position());
        self.reader.set_position(read_data.1);
        read_data.0
    }
}

impl<'a, E: Any> Data<'a> for ReadStream<'a, E> {
    fn setup<Sys: System<'a>>(store: &mut Store) {
        if store.get_resource::<EventStream<E>>().is_none() {
            store.insert_resource::<EventStream<E>>(EventStream::<E>::init());
        }
        if store.get_resource::<Reader<E, Sys>>().is_none() {
            store.insert_resource::<Reader<E, Sys>>(Reader::<E, Sys>::init());
        }
    }

    fn fetch<Sys: System<'a>>(store: &'a Store) -> Self {
        let reader = store
            .get_resource_mut::<Reader<E, Sys>>()
            .unwrap_or_else(|| {
                panic!(
                    "An error occurred during the fetch of the reader. ResourceId: {:?}",
                    TypeId::of::<Reader<E, Sys>>()
                )
            });
        trace!("fetched reader: {:?}", TypeId::of::<Reader<E, Sys>>());
        let stream = store.get_resource::<EventStream<E>>().unwrap_or_else(|| {
            panic!(
                "An error occurred during the fetch of the eventStream. ResourceId: {:?}",
                TypeId::of::<E>()
            )
        });

        ReadStream {
            stream: stream,
            reader: reader,
        }
    }
}

impl<'a, E: Any> Data<'a> for WriteStream<'a, E> {
    fn setup<Sys: System<'a>>(store: &mut Store) {
        if store.get_resource::<EventStream<E>>().is_none() {
            store.insert_resource::<EventStream<E>>(EventStream::<E>::init());
        }
        if store.get_resource::<Reader<E, Sys>>().is_none() {
            store.insert_resource::<Reader<E, Sys>>(Reader::<E, Sys>::init());
        }
    }

    fn fetch<Sys: System<'a>>(store: &'a Store) -> Self {
        let reader = store
            .get_resource_mut::<Reader<E, Sys>>()
            .unwrap_or_else(|| {
                panic!(
                    "An error occurred during the fetch of the resource. ResourceId: {:?}",
                    TypeId::of::<Reader<E, Sys>>()
                )
            });
        let stream = store
            .get_resource_mut::<EventStream<E>>()
            .unwrap_or_else(|| {
                panic!(
                    "An error occurred during the fetch of the resource. ResourceId: {:?}",
                    TypeId::of::<E>()
                )
            });

        WriteStream {
            stream: stream,
            reader: reader,
        }
    }
}

pub type ReadEntities<'a> = &'a Entities;
pub type Read<'a, R> = Ref<'a, R>;
pub type Write<'a, R> = RefMut<'a, R>;
pub type ReadOption<'a, R> = Option<Ref<'a, R>>;
pub type WriteOption<'a, R> = Option<RefMut<'a, R>>;
pub type ReadSet<'a, C> = Ref<'a, Set<C>>;
pub type WriteSet<'a, C> = RefMut<'a, Set<C>>;

impl<'a> Data<'a> for ReadEntities<'a> {
    fn setup<Sys: System<'a>>(store: &mut Store) {}

    fn fetch<Sys: System<'a>>(store: &'a Store) -> Self {
        store.get_entities()
    }
}

impl<'a, R: Resource + Default> Data<'a> for Read<'a, R> {
    fn setup<Sys: System<'a>>(store: &mut Store) {
        if store.get_resource::<R>().is_none() {
            store.insert_resource::<R>(R::default())
        }
    }

    fn fetch<Sys: System<'a>>(store: &'a Store) -> Self {
        store.get_resource::<R>().unwrap_or_else(|| {
            panic!(
                "An error occurred during the fetch of the resource. ResourceId: {:?}",
                TypeId::of::<R>()
            )
        })
    }
}

impl<'a, R: Resource + Default> Data<'a> for Write<'a, R> {
    fn setup<Sys: System<'a>>(store: &mut Store) {
        if store.get_resource::<R>().is_none() {
            store.insert_resource::<R>(R::default())
        }
    }

    fn fetch<Sys: System<'a>>(store: &'a Store) -> Self {
        store.get_resource_mut::<R>().unwrap_or_else(|| {
            panic!(
                "An error occurred during the fetch of the resource. ResourceId: {:?}",
                TypeId::of::<R>()
            )
        })
    }
}

impl<'a, R: Resource> Data<'a> for ReadOption<'a, R> {
    fn setup<Sys: System<'a>>(store: &mut Store) {}

    fn fetch<Sys: System<'a>>(store: &'a Store) -> Self {
        store.get_resource::<R>()
    }
}

impl<'a, R: Resource> Data<'a> for WriteOption<'a, R> {
    fn setup<Sys: System<'a>>(store: &mut Store) {}

    fn fetch<Sys: System<'a>>(store: &'a Store) -> Self {
        store.get_resource_mut::<R>()
    }
}

impl<'a, C: Component> Data<'a> for ReadSet<'a, C> {
    fn setup<Sys: System<'a>>(store: &mut Store) {
        store.register_component::<C>()
    }

    fn fetch<Sys: System<'a>>(store: &'a Store) -> Self {
        store.get_components::<C>().unwrap_or_else(|| {
            panic!(
                "An error occurred during the fetch of the component set. ComponentId: {:?}",
                TypeId::of::<C>()
            )
        })
    }
}

impl<'a, C: Component> Data<'a> for WriteSet<'a, C> {
    fn setup<Sys: System<'a>>(store: &mut Store) {
        store.register_component::<C>()
    }

    fn fetch<Sys: System<'a>>(store: &'a Store) -> Self {
        store.get_components_mut::<C>().unwrap_or_else(|| {
            panic!(
                "An error occurred during the fetch of the component set. ComponentId: {:?}",
                TypeId::of::<C>()
            )
        })
    }
}

macro_rules! impl_data {
    ( $($ty:ident),* ) => {
        impl<'a, $($ty),*> Data<'a> for ( $( $ty, )* )
            where $( $ty: Data<'a> ),*
            {
                fn setup<Sys: System<'a>>(store: &mut Store) {
                    $( $ty::setup::<Sys>(store); )*
                }

                fn fetch<Sys: System<'a>>(store: &'a Store) -> Self {
                    ( $( $ty::fetch::<Sys>(store), )* )
                }
            }
        }
}
impl_data!(A);
impl_data!(A, B);
impl_data!(A, B, C);
impl_data!(A, B, C, D);
impl_data!(A, B, C, D, E);
impl_data!(A, B, C, D, E, F);
impl_data!(A, B, C, D, E, F, G);
impl_data!(A, B, C, D, E, F, G, H);
impl_data!(A, B, C, D, E, F, G, H, I);
impl_data!(A, B, C, D, E, F, G, H, I, J);
impl_data!(A, B, C, D, E, F, G, H, I, J, K);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);

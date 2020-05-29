use crate::core::component::Component;
use crate::core::component::Set;
use crate::core::entity::EntitiesResource;
use crate::core::store::Resource;
use crate::core::store::Store;
use std::any::Any;
use std::cell::Ref;
use std::cell::RefMut;
use std::fmt::Debug;

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
        self.init(store);
    }

    fn run_now(&mut self, store: &Store) {
        let data = S::Data::get(store);
        self.run(data);
    }

    fn dispose(&mut self, store: &mut Store) {
        self.dispose(store);
    }
}

pub trait System<'a>: Any + Debug {
    type Data: Data<'a>;

    #[allow(unused_variables)]
    fn init(&mut self, store: &mut Store) {}

    fn run(&mut self, data: Self::Data);

    #[allow(unused_variables)]
    fn dispose(&mut self, store: &mut Store) {}
}

pub trait Data<'a> {
    fn get(store: &'a Store) -> Self;
}

impl<'a> Data<'a> for () {
    #[allow(unused_variables)]
    fn get(store: &Store) -> Self {
        ()
    }
}

pub type Entities<'a> = &'a EntitiesResource;
pub type Read<'a, R> = Ref<'a, R>;
pub type Write<'a, R> = RefMut<'a, R>;
pub type ReadSet<'a, C> = Ref<'a, Set<C>>;
pub type WriteSet<'a, C> = RefMut<'a, Set<C>>;

impl<'a> Data<'a> for Entities<'a> {
    fn get(store: &'a Store) -> Self {
        store.get_entities()
    }
}

impl<'a, R: Resource> Data<'a> for Read<'a, R> {
    fn get(store: &'a Store) -> Self {
        store.get_resource::<R>().unwrap()
    }
}

impl<'a, R: Resource> Data<'a> for Write<'a, R> {
    fn get(store: &'a Store) -> Self {
        store.get_resource_mut::<R>().unwrap()
    }
}

impl<'a, C: Component> Data<'a> for ReadSet<'a, C> {
    fn get(store: &'a Store) -> Self {
        store.get_components::<C>().unwrap()
    }
}

impl<'a, C: Component> Data<'a> for WriteSet<'a, C> {
    fn get(store: &'a Store) -> Self {
        store.get_components_mut::<C>().unwrap()
    }
}

macro_rules! impl_data {
    ( $($ty:ident),* ) => {
        impl<'a, $($ty),*> Data<'a> for ( $( $ty, )* )
            where $( $ty: Data<'a> ),*
            {
                fn get(store: &'a Store) -> Self {
                    ( $( $ty::get(store), )* )
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

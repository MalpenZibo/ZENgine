use crate::core::component::Component;
use crate::core::component::Set;
use crate::core::store::Store;
use std::any::Any;
use std::cell::Ref;
use std::cell::RefMut;
use std::fmt::Debug;

pub trait System: Any + Debug {
    #[allow(unused_variables)]
    fn init(&mut self, store: &mut Store) {}

    fn run(&mut self, store: &Store);

    #[allow(unused_variables)]
    fn dispose(&mut self, store: &mut Store) {}
}

pub trait Data<'a> {
    fn get(store: &'a Store) -> Self;
}

pub type ReadSet<'a, C> = Ref<'a, Set<C>>;
pub type WriteSet<'a, C> = RefMut<'a, Set<C>>;

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

#[macro_export]
macro_rules! unpack {
    ( $store:expr, $($ty:ident<$r:ident>),* ) => {
        ( $( <$ty<$r> as Data>::get($store), )* )
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Test {
        data: u32,
    }
    impl Component for Test {}

    #[derive(Debug)]
    struct Test2 {
        data2: u32,
    }
    impl Component for Test2 {}

    #[test]
    fn test_unpack() {
        let store = Store::default();

        let (test1, test2) = unpack!(&store, ReadSet<Test>, ReadSet<Test2>);

        println!("test 1 {:?}", test1);
        println!("test 2 {:?}", test2);
    }
}

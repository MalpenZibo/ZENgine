use crate::core::component::AnySet;
use crate::core::component::Component;
use crate::core::component::Set;
use crate::core::entity::Entity;
use std::collections::hash_map::Iter;
use std::collections::hash_map::IterMut;
use std::collections::hash_map::Keys;
use std::marker::PhantomData;

enum JoinReturn<T> {
    Skip,
    Value(T),
}

trait Joined {
    type Output;

    fn get_join_value(&mut self, entity: &Entity) -> JoinReturn<Self::Output>;
}

impl<'a, C: Component> Joined for &'a Set<C> {
    type Output = &'a C;

    fn get_join_value(&mut self, entity: &Entity) -> JoinReturn<Self::Output> {
        match self.get(entity) {
            Some(component) => JoinReturn::Value(component),
            None => JoinReturn::Skip,
        }
    }
}

impl<'a, C: Component> Joined for &'a mut Set<C> {
    type Output = &'a mut C;

    fn get_join_value(&mut self, entity: &Entity) -> JoinReturn<Self::Output> {
        //SAFETY FIXME write something that explains why this unsafe code
        //is actually safe
        unsafe {
            match self.get_mut(entity) {
                Some(component) => JoinReturn::Value(&mut *(component as *mut C)),
                None => JoinReturn::Skip,
            }
        }
    }
}

trait Joinable {}
impl<T: Joined> Joinable for T {}

trait Join<C: Component, D> {
    fn join(&self, joined: D) -> JoinIter<Iter<Entity, C>, D>;

    fn join_mut(&mut self, joined: D) -> JoinIter<IterMut<Entity, C>, D>;
}

impl<Comp: Component, D: Joinable> Join<Comp, D> for Set<Comp> {
    fn join(&self, joined: D) -> JoinIter<Iter<Entity, Comp>, D> {
        JoinIter {
            iter: self.iter(),
            joined: joined,
        }
    }

    fn join_mut(&mut self, joined: D) -> JoinIter<IterMut<Entity, Comp>, D> {
        JoinIter {
            iter: self.iter_mut(),
            joined: joined,
        }
    }
}

macro_rules! impl_join_for_tuple {
    ( $($ty:ident),* ) => {
        impl<Comp: Component, $($ty),*> Join<Comp, ( $( $ty, )* )> for Set<Comp>
            where $( $ty: Joinable ),*
            {
                fn join(&self, joined: ( $( $ty, )* )) -> JoinIter<Iter<Entity, Comp>, ( $( $ty, )* )> {
                    JoinIter {
                        iter: self.iter(),
                        joined: joined,
                    }
                }
                fn join_mut(&mut self, joined: ( $( $ty, )* )) -> JoinIter<IterMut<Entity, Comp>, ( $( $ty, )* )> {
                    JoinIter {
                        iter: self.iter_mut(),
                        joined: joined,
                    }
                }
            }
        }
}
impl_join_for_tuple!(A, B);
impl_join_for_tuple!(A, B, C);
impl_join_for_tuple!(A, B, C, D);
impl_join_for_tuple!(A, B, C, D, E);
impl_join_for_tuple!(A, B, C, D, E, F);
impl_join_for_tuple!(A, B, C, D, E, F, G);
impl_join_for_tuple!(A, B, C, D, E, F, G, H);
impl_join_for_tuple!(A, B, C, D, E, F, G, H, I);
impl_join_for_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_join_for_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_join_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_join_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_join_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_join_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_join_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_join_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_join_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_join_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_join_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_join_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
impl_join_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
impl_join_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
impl_join_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
impl_join_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
impl_join_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);

struct JoinIter<I: Iterator, D> {
    iter: I,
    joined: D,
}

impl<'a, A: 'a + Component, B: 'a + Joined> Iterator for JoinIter<Iter<'a, Entity, A>, B> {
    type Item = (&'a Entity, &'a A, B::Output);

    fn next(&mut self) -> Option<Self::Item> {
        let mut next_value: Option<Self::Item> = None;
        while let Some(entry) = self.iter.next() {
            next_value = Some((
                entry.0,
                entry.1,
                match self.joined.get_join_value(entry.0) {
                    JoinReturn::Value(other) => other,
                    JoinReturn::Skip => break,
                },
            ));

            if next_value.is_some() {
                break;
            }
        }

        next_value
    }
}

macro_rules! impl_join_iterator_for_tuple {
    ( $($ty:ident => $index:tt),* ) => {
        impl<'a, Comp, $($ty),*> Iterator
            for JoinIter<Iter<'a, Entity, Comp>, ( $( $ty, )* )>
            where Comp: 'a + Component, $( $ty: 'a + Joined ),*
        {
            type Item = (&'a Entity, &'a Comp, $( $ty::Output, )* );

            fn next(&mut self) -> Option<Self::Item> {
                let mut next_value: Option<Self::Item> = None;
                while let Some(entry) = self.iter.next() {
                    next_value = Some((
                        entry.0,
                        entry.1,
                        $(
                            match self.joined.$index.get_join_value(entry.0) {
                                JoinReturn::Value(other) => other,
                                JoinReturn::Skip => break,
                            }
                        ), *
                    ));

                    if next_value.is_some() {
                        break;
                    }
                }

                next_value
            }
        }
    }
}
impl_join_iterator_for_tuple!(A => 0, B => 1);
//impl_join_iterator_for_tuple!(A => [0], [mut], B => [1], [mut], C => [2], [mut]);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::store::Store;

    #[derive(Component, PartialEq, Debug)]
    struct Component1 {
        data1: i32,
        data2: f32,
    }

    #[derive(Component, PartialEq, Debug)]
    struct Component2 {
        data3: String,
        data4: u32,
    }

    #[derive(Component, PartialEq, Debug)]
    struct Component3 {
        data5: i32,
    }

    fn prapare_join_test() -> (Set<Component1>, Set<Component2>, Set<Component3>) {
        let mut store = Store::default();
        let entity1 = store.build_entity().build();
        let entity2 = store.build_entity().build();
        let entity3 = store.build_entity().build();
        let entity4 = store.build_entity().build();
        let mut storage1: Set<Component1> = Set::default();
        let mut storage2: Set<Component2> = Set::default();
        let mut storage3: Set<Component3> = Set::default();

        storage1.insert(
            entity1,
            Component1 {
                data1: 3,
                data2: 3.5,
            },
        );

        storage1.insert(
            entity2,
            Component1 {
                data1: 6,
                data2: 7.8,
            },
        );

        storage1.insert(
            entity3,
            Component1 {
                data1: 9,
                data2: 2.1,
            },
        );

        storage1.insert(
            entity4,
            Component1 {
                data1: 3,
                data2: 3.5,
            },
        );

        storage2.insert(
            entity1,
            Component2 {
                data3: "test".to_string(),
                data4: 2,
            },
        );

        storage2.insert(
            entity2,
            Component2 {
                data3: "test2".to_string(),
                data4: 7,
            },
        );

        storage3.insert(entity1, Component3 { data5: 5 });

        storage3.insert(entity2, Component3 { data5: 5 });

        storage3.insert(entity3, Component3 { data5: 5 });

        storage3.insert(entity4, Component3 { data5: 5 });

        (storage1, storage2, storage3)
    }
    #[test]
    fn join_iterator() {
        let (mut storage1, mut storage2, mut storage3) = prapare_join_test();

        let join_iter = storage1.join(&storage2);
        let join_iter = storage1.join(&mut storage2);
        let join_iter = storage1.join((&storage2, &storage3));
        let join_iter = storage1.join((&mut storage2, &storage3));
        let join_iter = storage1.join((&storage2, &mut storage3));
        let join_iter = storage1.join((&mut storage2, &mut storage3));

        assert_eq!(storage1.join(&storage2).count(), 2);

        for (entity, c1, c2) in storage1.join(&storage2) {
            println!("{:?}", c1.data1);
        }

        assert_eq!(storage1.join((&storage2, &storage3)).count(), 2);

        for (entity, c1, c2, c3) in storage1.join((&storage2, &storage3)) {
            println!("{:?}", c1.data1);
        }

        for (entity, c1, mut c2) in storage1.join(&mut storage2) {
            println!("{:?}", c1.data1);
            c2.data4 = 7;
        }

        for (entity, c1, c2, c3) in storage1.join((&storage2, &storage3)) {
            println!("{:?}", c1.data1);
        }

        //let test = storage1.join(5);
    }

    /*
    #[test]
    fn join_iterator_mut() {
        let (mut storage1, mut storage2, mut storage3) = prapare_join_test();

        //assert_eq!((&mut storage1, &storage2, &mut storage3).join(), 2);

        let test = (&mut storage1, &storage2, &mut storage3).join();

        for (entity, mut c1, c2, mut c3) in (&mut storage1, &storage2, &mut storage3).join() {
            println!("{:?}", c1.data1);
            c1.data1 = 5;
            c3.data5 = 7;
        }
    }*/
}

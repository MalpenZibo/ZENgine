use crate::core::component::Component;
use crate::core::component::Set;
use crate::core::entity::Entity;
use std::collections::hash_map::Iter;
use std::collections::hash_map::IterMut;

enum JoinReturn<T> {
    Skip,
    Value(T),
}

struct Optional<T: Joinable>(T);

trait Joined {
    type Output;

    #[allow(clippy::trivially_copy_pass_by_ref)]
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

impl<'a, C: Component> Joined for Optional<&'a Set<C>> {
    type Output = Option<&'a C>;

    fn get_join_value(&mut self, entity: &Entity) -> JoinReturn<Self::Output> {
        JoinReturn::Value(self.0.get(entity))
    }
}

impl<'a, C: Component> Joined for Optional<&'a mut Set<C>> {
    type Output = Option<&'a mut C>;

    fn get_join_value(&mut self, entity: &Entity) -> JoinReturn<Self::Output> {
        //SAFETY FIXME write something that explains why this unsafe code
        //is actually safe
        unsafe {
            JoinReturn::Value(match self.0.get_mut(entity) {
                Some(component) => Some(&mut *(component as *mut C)),
                None => None,
            })
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
            joined,
        }
    }

    fn join_mut(&mut self, joined: D) -> JoinIter<IterMut<Entity, Comp>, D> {
        JoinIter {
            iter: self.iter_mut(),
            joined,
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

impl<'a, A: 'a + Component, B: 'a + Joined> Iterator for JoinIter<IterMut<'a, Entity, A>, B> {
    type Item = (&'a Entity, &'a mut A, B::Output);

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
impl_join_iterator_for_tuple!(A => 0, B => 1, C => 2);
impl_join_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3);
impl_join_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4);
impl_join_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5);
impl_join_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6);
impl_join_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7);
impl_join_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8);
impl_join_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9);
impl_join_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10);
impl_join_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11);
impl_join_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12);
impl_join_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13);
impl_join_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14);
impl_join_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15);
impl_join_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16);
impl_join_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17);
impl_join_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17, S => 18);
impl_join_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17, S => 18, T => 19);
impl_join_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17, S => 18, T => 19, U => 20);
impl_join_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17, S => 18, T => 19, U => 20, V => 21);
impl_join_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17, S => 18, T => 19, U => 20, V => 21, W => 22);
impl_join_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17, S => 18, T => 19, U => 20, V => 21, W => 22, X => 23);
impl_join_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17, S => 18, T => 19, U => 20, V => 21, W => 22, X => 23, Y => 24);
impl_join_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17, S => 18, T => 19, U => 20, V => 21, W => 22, X => 23, Y => 24, Z => 25 );

macro_rules! impl_join_mut_iterator_for_tuple {
    ( $($ty:ident => $index:tt),* ) => {
        impl<'a, Comp, $($ty),*> Iterator
            for JoinIter<IterMut<'a, Entity, Comp>, ( $( $ty, )* )>
            where Comp: 'a + Component, $( $ty: 'a + Joined ),*
        {
            type Item = (&'a Entity, &'a mut Comp, $( $ty::Output, )* );

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
impl_join_mut_iterator_for_tuple!(A => 0, B => 1);
impl_join_mut_iterator_for_tuple!(A => 0, B => 1, C => 2);
impl_join_mut_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3);
impl_join_mut_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4);
impl_join_mut_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5);
impl_join_mut_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6);
impl_join_mut_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7);
impl_join_mut_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8);
impl_join_mut_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9);
impl_join_mut_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10);
impl_join_mut_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11);
impl_join_mut_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12);
impl_join_mut_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13);
impl_join_mut_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14);
impl_join_mut_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15);
impl_join_mut_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16);
impl_join_mut_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17);
impl_join_mut_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17, S => 18);
impl_join_mut_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17, S => 18, T => 19);
impl_join_mut_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17, S => 18, T => 19, U => 20);
impl_join_mut_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17, S => 18, T => 19, U => 20, V => 21);
impl_join_mut_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17, S => 18, T => 19, U => 20, V => 21, W => 22);
impl_join_mut_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17, S => 18, T => 19, U => 20, V => 21, W => 22, X => 23);
impl_join_mut_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17, S => 18, T => 19, U => 20, V => 21, W => 22, X => 23, Y => 24);
impl_join_mut_iterator_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17, S => 18, T => 19, U => 20, V => 21, W => 22, X => 23, Y => 24, Z => 25 );

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
    fn join_iterator_global() {
        let (mut storage1, mut storage2, mut storage3) = prapare_join_test();

        let _join_iter = storage1.join(&storage2);
        let _join_iter = storage1.join(&mut storage2);
        let _join_iter = storage1.join((&storage2, &storage3));
        let _join_iter = storage1.join((&mut storage2, &storage3));
        let _join_iter = storage1.join((&storage2, &mut storage3));
        let _join_iter = storage1.join((&mut storage2, &mut storage3));

        assert_eq!(storage1.join(&storage2).count(), 2);
        assert_eq!(storage1.join(Optional(&storage2)).count(), 4);
        assert_eq!(storage1.join(Optional(&mut storage2)).count(), 4);

        assert_eq!(storage1.join((&storage3, Optional(&storage2))).count(), 4);
        assert_eq!(
            storage1.join((Optional(&mut storage2), &storage3)).count(),
            4
        );

        for (_entity, c1, _c2) in storage1.join(&storage2) {
            println!("{:?}", c1.data1);
        }

        assert_eq!(storage1.join((&storage2, &storage3)).count(), 2);

        for (_entity, c1, c2, c3) in storage1.join((&storage2, &storage3)) {
            println!("{:?}, {:?}, {:?}", c1.data1, c2.data3, c3.data5);
        }

        for (_entity, c1, mut c2) in storage1.join(&mut storage2) {
            println!("{:?}", c1.data1);
            c2.data4 = 7;
        }

        for (_entity, c1, c2, mut c3) in storage1.join((&storage2, &mut storage3)) {
            println!("{:?}, {:?}, {:?}", c1.data1, c2.data3, c3.data5);
            c3.data5 = 7;
        }

        for (_entity, mut c1, c2, mut c3) in storage1.join_mut((&storage2, &mut storage3)) {
            println!("{:?}, {:?}, {:?}", c1.data1, c2.data3, c3.data5);
            c1.data1 = 3;
            c3.data5 = 89;
        }
    }

    #[test]
    fn join_iterator() {
        let (mut storage1, mut storage2, mut storage3) = prapare_join_test();
        assert_eq!(storage1.join(&storage2).count(), 2);
        assert_eq!(storage1.join(&mut storage2).count(), 2);
        assert_eq!(storage1.join(&storage3).count(), 4);
        assert_eq!(storage1.join(&mut storage3).count(), 4);

        assert_eq!(storage1.join_mut(&storage2).count(), 2);
        assert_eq!(storage1.join_mut(&mut storage2).count(), 2);
        assert_eq!(storage1.join_mut(&storage3).count(), 4);
        assert_eq!(storage1.join_mut(&mut storage3).count(), 4);
    }

    #[test]
    fn join_optional_iterator() {
        let (mut storage1, mut storage2, _storage3) = prapare_join_test();
        assert_eq!(storage1.join(Optional(&storage2)).count(), 4);
        assert_eq!(storage1.join(Optional(&mut storage2)).count(), 4);

        assert_eq!(storage1.join_mut(Optional(&storage2)).count(), 4);
        assert_eq!(storage1.join_mut(Optional(&mut storage2)).count(), 4);
    }

    #[test]
    fn join_tuple_iterator() {
        let (mut storage1, mut storage2, mut storage3) = prapare_join_test();
        assert_eq!(storage1.join((&storage2, &storage3)).count(), 2);
        assert_eq!(storage1.join((&mut storage2, &storage3)).count(), 2);
        assert_eq!(storage1.join((&storage2, &storage3)).count(), 2);
        assert_eq!(storage1.join((&storage2, &mut storage3)).count(), 2);
        assert_eq!(storage1.join((&mut storage2, &mut storage3)).count(), 2);

        assert_eq!(storage1.join_mut((&storage2, &storage3)).count(), 2);
        assert_eq!(storage1.join_mut((&mut storage2, &storage3)).count(), 2);
        assert_eq!(storage1.join_mut((&storage2, &storage3)).count(), 2);
        assert_eq!(storage1.join_mut((&storage2, &mut storage3)).count(), 2);
        assert_eq!(storage1.join_mut((&mut storage2, &mut storage3)).count(), 2);
    }

    #[test]
    fn join_tuple_optional_iterator() {
        let (mut storage1, mut storage2, mut storage3) = prapare_join_test();
        assert_eq!(storage1.join((Optional(&storage2), &storage3)).count(), 4);
        assert_eq!(
            storage1.join((Optional(&mut storage2), &storage3)).count(),
            4
        );

        assert_eq!(storage1.join((&storage2, Optional(&storage3))).count(), 2);
        assert_eq!(
            storage1.join((&storage2, Optional(&mut storage3))).count(),
            2
        );

        assert_eq!(
            storage1.join_mut((Optional(&storage2), &storage3)).count(),
            4
        );
        assert_eq!(
            storage1
                .join_mut((Optional(&mut storage2), &storage3))
                .count(),
            4
        );

        assert_eq!(
            storage1.join_mut((&storage2, Optional(&storage3))).count(),
            2
        );
        assert_eq!(
            storage1
                .join_mut((&storage2, Optional(&mut storage3)))
                .count(),
            2
        );
    }
}

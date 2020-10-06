use crate::core::component::Component;
use crate::core::component::Set;
use crate::core::entity::Entity;
use std::collections::hash_map::Iter;
use std::collections::hash_map::IterMut;

pub enum JoinComponentReturn<D> {
    Skip,
    Value(D),
}

pub struct JoinIterator<I, D> {
    iter: I,
    other: D,
}

pub trait Joined {
    type Iter;
    type Item;

    fn iter(self) -> Self::Iter;

    fn entity_component(self, entity: &Entity) -> JoinComponentReturn<Self::Item>;
}

type JoinedSet<'a, C: Component> = &'a Set<C>;
type JoinedSetMut<'a, C: Component> = &'a mut Set<C>;

impl<'a, C: Component> Joined for JoinedSet<'a, C> {
    type Iter = Iter<'a, Entity, C>;
    type Item = &'a C;

    fn iter(self) -> Self::Iter {
        self.iter()
    }

    fn entity_component(self, entity: &Entity) -> JoinComponentReturn<Self::Item> {
        match self.get(entity) {
            Some(component) => JoinComponentReturn::Value(component),
            None => JoinComponentReturn::Skip,
        }
    }
}

impl<'a, C: Component> Joined for JoinedSetMut<'a, C> {
    type Iter = IterMut<'a, Entity, C>;
    type Item = &'a mut C;

    fn iter(self) -> Self::Iter {
        self.iter_mut()
    }

    fn entity_component(self, entity: &Entity) -> JoinComponentReturn<Self::Item> {
        match self.get_mut(entity) {
            Some(component) => JoinComponentReturn::Value(component),
            None => JoinComponentReturn::Skip,
        }
    }
}

impl<'a, A: Component, B: Joined + Copy> Iterator for JoinIterator<Iter<'a, Entity, A>, B> {
    type Item = (&'a Entity, &'a A, B::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let mut next_value: Option<Self::Item> = None;

        while let Some(entry) = self.iter.next() {
            let c1 = match self.other.entity_component(entry.0) {
                JoinComponentReturn::Skip => {
                    break;
                }
                JoinComponentReturn::Value(component) => component,
            };

            next_value = Some((entry.0, entry.1, c1));
            break;
        }

        next_value
    }
}

impl<'a, A: Component, B: Joined + Copy> Iterator for JoinIterator<IterMut<'a, Entity, A>, B> {
    type Item = (&'a Entity, &'a mut A, B::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let mut next_value: Option<Self::Item> = None;

        while let Some(entry) = self.iter.next() {
            let c1 = match self.other.entity_component(entry.0) {
                JoinComponentReturn::Skip => {
                    break;
                }
                JoinComponentReturn::Value(component) => component,
            };

            next_value = Some((entry.0, entry.1, c1));
            break;
        }

        next_value
    }
}

pub trait Joinable<I, D> {
    fn join(self) -> JoinIterator<I, D>;
}

impl<'a, A: Component, B: Component> Joinable<Iter<'a, Entity, A>, JoinedSet<'a, B>>
    for (JoinedSet<'a, A>, JoinedSet<'a, B>)
{
    fn join(self) -> JoinIterator<Iter<'a, Entity, A>, JoinedSet<'a, B>> {
        JoinIterator {
            iter: self.0.iter(),
            other: self.1,
        }
    }
}

impl<'a, A: Component, B: Component> Joinable<IterMut<'a, Entity, A>, JoinedSet<'a, B>>
    for (JoinedSetMut<'a, A>, JoinedSet<'a, B>)
{
    fn join(self) -> JoinIterator<IterMut<'a, Entity, A>, JoinedSet<'a, B>> {
        JoinIterator {
            iter: self.0.iter_mut(),
            other: self.1,
        }
    }
}

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
        let (storage1, storage2, storage3) = prapare_join_test();

        assert_eq!((&storage1, &storage2).join().count(), 2);

        for (entity, c1, c2) in (&storage1, &storage2).join() {
            println!("{:?}", c1.data1);
        }
    }
    #[test]
    fn join_iterator_mut() {
        let (mut storage1, mut storage2, mut storage3) = prapare_join_test();

        assert_eq!((&mut storage1, &storage2).join().count(), 2);

        for (entity, mut c1, c2) in (&mut storage1, &storage2).join() {
            println!("{:?}", c1.data1);
            c1.data1 = 5;
            //c2.data4 = 7;
        }
    }
}

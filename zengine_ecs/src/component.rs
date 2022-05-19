use std::{
    any::{Any, TypeId},
    fmt::Debug,
    sync::RwLock,
};

use crate::archetype::Archetype;

pub trait ComponentBundle {
    fn get_types() -> Vec<TypeId>;

    fn get_component_columns() -> Vec<Box<dyn ComponentColumn>>;

    fn inser_into(archetype: &mut Archetype, bundle: Self, columns: Vec<usize>);

    fn replace_into(archetype: &mut Archetype, bundle: Self, row: usize, columns: Vec<usize>);
}

pub trait Component: Any + Debug {}

impl<T: Component> ComponentBundle for T {
    fn get_types() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn get_component_columns() -> Vec<Box<dyn ComponentColumn>> {
        vec![Box::new(RwLock::new(Vec::<T>::new()))]
    }

    fn inser_into(archetype: &mut Archetype, bundle: Self, mut columns: Vec<usize>) {
        let column_index = columns.pop().unwrap();
        let column = component_vec_to_mut(&mut *archetype.components[column_index]);
        column.push(bundle);
    }

    fn replace_into(archetype: &mut Archetype, bundle: Self, row: usize, mut columns: Vec<usize>) {
        let column_index = columns.pop().unwrap();
        let column = component_vec_to_mut(&mut *archetype.components[column_index]);
        column[row] = bundle;
    }
}

impl<A: Component, B: Component> ComponentBundle for (A, B) {
    fn get_types() -> Vec<TypeId> {
        vec![TypeId::of::<A>(), TypeId::of::<B>()]
    }

    fn get_component_columns() -> Vec<Box<dyn ComponentColumn>> {
        vec![
            Box::new(RwLock::new(Vec::<A>::new())),
            Box::new(RwLock::new(Vec::<B>::new())),
        ]
    }

    fn inser_into(archetype: &mut Archetype, bundle: Self, mut columns: Vec<usize>) {
        let (a, b) = bundle;

        let column_index = columns.pop().unwrap();
        let column = component_vec_to_mut::<B>(&mut *archetype.components[column_index]);
        column.push(b);

        let column_index = columns.pop().unwrap();
        let column = component_vec_to_mut::<A>(&mut *archetype.components[column_index]);
        column.push(a);
    }

    fn replace_into(archetype: &mut Archetype, bundle: Self, row: usize, mut columns: Vec<usize>) {
        let (a, b) = bundle;

        let e = columns.pop().unwrap();
        let column = component_vec_to_mut::<B>(&mut *archetype.components[e]);
        column[row] = b;

        let e = columns.pop().unwrap();
        let column = component_vec_to_mut::<A>(&mut *archetype.components[e]);
        column[row] = a;
    }
}

pub trait ComponentColumn: Debug {
    fn to_any(&self) -> &dyn Any;
    fn to_any_mut(&mut self) -> &mut dyn Any;
    fn swap_remove(&mut self, row_index: usize) -> Box<dyn Component>;
    fn new_same_type(&self) -> Box<dyn ComponentColumn>;
    fn migrate(&mut self, row_index: usize, other_component_vec: &mut dyn ComponentColumn);
}

impl<T: Component> ComponentColumn for RwLock<Vec<T>> {
    fn to_any(&self) -> &dyn Any {
        self
    }
    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn swap_remove(&mut self, row_index: usize) -> Box<dyn Component> {
        Box::new(self.get_mut().unwrap().swap_remove(row_index))
    }

    fn new_same_type(&self) -> Box<dyn ComponentColumn> {
        Box::new(RwLock::new(Vec::<T>::new()))
    }

    fn migrate(&mut self, row_index: usize, other_component_vec: &mut dyn ComponentColumn) {
        let data: T = self.get_mut().unwrap().swap_remove(row_index);
        component_vec_to_mut(other_component_vec).push(data);
    }
}

pub fn component_vec_to_mut<T: Component>(c: &mut dyn ComponentColumn) -> &mut Vec<T> {
    c.to_any_mut()
        .downcast_mut::<RwLock<Vec<T>>>()
        .expect("donwcasting error")
        .get_mut()
        .expect("lock error")
}

use std::{
    any::{Any, TypeId},
    fmt::Debug,
    sync::RwLock,
};

use crate::archetype::Archetype;

pub trait Component: Any + Sync + Send + Debug {}

pub trait ComponentBundle {
    fn get_types() -> Vec<TypeId>;

    fn get_component_columns() -> Vec<Box<dyn ComponentColumn>>;

    fn inser_into(self, archetype: &mut Archetype, columns: Vec<usize>);

    fn replace_into(self, archetype: &mut Archetype, row: usize, columns: Vec<usize>);
}

impl<T: Component> ComponentBundle for T {
    fn get_types() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn get_component_columns() -> Vec<Box<dyn ComponentColumn>> {
        vec![Box::new(RwLock::new(Vec::<T>::new()))]
    }

    fn inser_into(self, archetype: &mut Archetype, columns: Vec<usize>) {
        let column_index = columns[0];
        let column = component_vec_to_mut(&mut *archetype.components[column_index]);
        column.push(self);
    }

    fn replace_into(self, archetype: &mut Archetype, row: usize, columns: Vec<usize>) {
        let column_index = columns[0];
        let column = component_vec_to_mut(&mut *archetype.components[column_index]);
        column[row] = self;
    }
}

macro_rules! impl_component_bundle_for_tuple {
    ( $($ty:ident => $index:tt),* ) => {
        impl<$($ty),*> ComponentBundle for ( $( $ty, )* )
        where $( $ty: Component ),*
        {
            fn get_types() -> Vec<TypeId> {
                vec![$( TypeId::of::<$ty>(), )*]
            }

            fn get_component_columns() -> Vec<Box<dyn ComponentColumn>> {
                vec![
                    $( Box::new(RwLock::new(Vec::<$ty>::new())), )*
                ]
            }

            fn inser_into(self, archetype: &mut Archetype, columns: Vec<usize>) {
                $(
                    let column_index = columns[$index];
                    let column = component_vec_to_mut::<$ty>(&mut *archetype.components[column_index]);
                    column.push(self.$index);
                ) *

            }

            fn replace_into(self, archetype: &mut Archetype, row: usize, columns: Vec<usize>) {
                $(
                    let column_index = columns[$index];
                    let column = component_vec_to_mut::<$ty>(&mut *archetype.components[column_index]);
                    column[row] = self.$index;
                ) *
            }
        }
    }
}
impl_component_bundle_for_tuple!(A => 0, B => 1);
impl_component_bundle_for_tuple!(A => 0, B => 1, C => 2);
impl_component_bundle_for_tuple!(A => 0, B => 1, C => 2, D => 3);
impl_component_bundle_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4);
impl_component_bundle_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5);
impl_component_bundle_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6);
impl_component_bundle_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7);
impl_component_bundle_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8);
impl_component_bundle_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9);
impl_component_bundle_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10);
impl_component_bundle_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11);
impl_component_bundle_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12);
impl_component_bundle_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13);
impl_component_bundle_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14);
impl_component_bundle_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15);
impl_component_bundle_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16);
impl_component_bundle_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17);
impl_component_bundle_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17, S => 18);
impl_component_bundle_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17, S => 18, T => 19);
impl_component_bundle_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17, S => 18, T => 19, U => 20);
impl_component_bundle_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17, S => 18, T => 19, U => 20, V => 21);
impl_component_bundle_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17, S => 18, T => 19, U => 20, V => 21, W => 22);
impl_component_bundle_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17, S => 18, T => 19, U => 20, V => 21, W => 22, X => 23);
impl_component_bundle_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17, S => 18, T => 19, U => 20, V => 21, W => 22, X => 23, Y => 24);
impl_component_bundle_for_tuple!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15, Q => 16, R => 17, S => 18, T => 19, U => 20, V => 21, W => 22, X => 23, Y => 24, Z => 25 );

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

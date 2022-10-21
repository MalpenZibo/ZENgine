use std::{
    any::{Any, TypeId},
    fmt::Debug,
    sync::RwLock,
};

use zengine_macro::all_positional_tuples;

use crate::archetype::Archetype;

/// A data type that can be used to store data for an [Entity](crate::Entity)
///
/// Component is a [derivable trait](https://doc.rust-lang.org/book/appendix-03-derivable-traits.html):
/// you could implement it by applying a `#[derive(Component)]` attribute to your data type.
/// To correctly implement this trait your data must satisfy the `Sync + Send + Debug` trait bounds.
///
/// # Implementing the trait for foreign types
/// As a consequence of the [orphan rule](https://doc.rust-lang.org/book/ch10-02-traits.html#implementing-a-trait-on-a-type),
/// it is not possible to separate into two different crates the implementation of Component
/// from the definition of a type.
/// For this reason is not possible to implement the Component trat for a type defined in a third party library.
/// The newtype pattern is a simple workaround to this limitation:
///
/// The following example gives a demonstration of this pattern.
/// ```ingore
/// use external_crate::TypeThatShouldBeAComponent;
/// use zengine_macro::Component;
///
/// #[derive(Component, Debug)]
/// struct MyWrapper(TypeThatShouldBeAComponent);
/// ```
pub trait Component: Any + Sync + Send + Debug {}

#[doc(hidden)]
pub enum InsertType {
    Add,
    Replace(usize),
}

#[doc(hidden)]
pub trait ComponentBundle {
    fn get_types() -> Vec<TypeId>;

    fn get_component_columns() -> Vec<(TypeId, Box<dyn ComponentColumn>)>;

    fn inser_into(self, archetype: &mut Archetype, columns: Vec<(InsertType, usize)>);
}

impl<T: Component> ComponentBundle for T {
    fn get_types() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn get_component_columns() -> Vec<(TypeId, Box<dyn ComponentColumn>)> {
        vec![(TypeId::of::<T>(), Box::new(RwLock::new(Vec::<T>::new())))]
    }

    fn inser_into(self, archetype: &mut Archetype, mut columns: Vec<(InsertType, usize)>) {
        let (insert_type, column_index) = columns.pop().expect("should have a value");
        let column = component_vec_to_mut(&mut *archetype.components[column_index]);
        if let InsertType::Replace(row) = insert_type {
            column[row] = self;
        } else {
            column.push(self);
        }
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

            fn get_component_columns() -> Vec<(TypeId, Box<dyn ComponentColumn>)> {
                vec![
                    $( (TypeId::of::<$ty>() ,Box::new(RwLock::new(Vec::<$ty>::new()))), )*
                ]
            }

            fn inser_into(self, _archetype: &mut Archetype, _columns: Vec<(InsertType, usize)>) {
                $(
                    let (insert_type, column_index) = _columns.get($index).unwrap();
                    let column = component_vec_to_mut::<$ty>(&mut *_archetype.components[*column_index]);
                    if let InsertType::Replace(row) = insert_type {
                        column[*row] = self.$index;
                    } else {
                        column.push(self.$index);
                    }
                ) *

            }
        }
    }
}
all_positional_tuples!(impl_component_bundle_for_tuple, 0, 14, C);

#[doc(hidden)]
pub trait ComponentColumn: Debug {
    fn to_any(&self) -> &dyn Any;
    fn to_any_mut(&mut self) -> &mut dyn Any;
    fn swap_remove(&mut self, row_index: usize) -> Box<dyn Component>;
    fn new_same_type(&self) -> (TypeId, Box<dyn ComponentColumn>);
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

    fn new_same_type(&self) -> (TypeId, Box<dyn ComponentColumn>) {
        (TypeId::of::<T>(), Box::new(RwLock::new(Vec::<T>::new())))
    }

    fn migrate(&mut self, row_index: usize, other_component_vec: &mut dyn ComponentColumn) {
        let data: T = self.get_mut().unwrap().swap_remove(row_index);
        component_vec_to_mut(other_component_vec).push(data);
    }
}

pub(crate) fn component_vec_to_mut<T: Component>(c: &mut dyn ComponentColumn) -> &mut Vec<T> {
    c.to_any_mut()
        .downcast_mut::<RwLock<Vec<T>>>()
        .expect("donwcasting error")
        .get_mut()
        .expect("lock error")
}

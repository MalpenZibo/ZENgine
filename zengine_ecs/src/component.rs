use std::{any::Any, sync::RwLock};

pub trait Component: Any {}

pub trait ComponentColumn {
    fn to_any(&self) -> &dyn Any;
    fn to_any_mut(&mut self) -> &mut dyn Any;
    fn swap_remove(&mut self, row_index: usize) -> Box<dyn Component>;
    fn new_same_type(&self) -> Box<dyn ComponentColumn>;
    fn push(&mut self, component: Box<dyn Component>);
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
    fn push(&mut self, component: Box<dyn Component>) {
        self.push(component)
    }

    fn new_same_type(&self) -> Box<dyn ComponentColumn> {
        Box::new(RwLock::new(Vec::<T>::new()))
    }
}

pub fn component_vec_to_mut<T: 'static>(c: &mut dyn ComponentColumn) -> &mut Vec<T> {
    c.to_any_mut()
        .downcast_mut::<Vec<T>>()
        .expect("donwcasting error")
}

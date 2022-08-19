use std::fmt::Debug;
use std::{any::Any, cell::RefCell, sync::RwLock};

pub trait Resource: Any + Sync + Send + Debug {}

pub trait ResourceCell: Debug {
    fn to_any(&self) -> &dyn Any;
    fn to_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Resource> ResourceCell for RwLock<T> {
    fn to_any(&self) -> &dyn Any {
        self
    }
    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub trait UnsendableResource: Any + Debug {}

pub trait UnsendableResourceCell: Debug {
    fn to_any(&self) -> &dyn Any;
    fn to_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: UnsendableResource> UnsendableResourceCell for RefCell<T> {
    fn to_any(&self) -> &dyn Any {
        self
    }
    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

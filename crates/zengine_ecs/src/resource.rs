use std::cell::{Ref, RefMut};
use std::fmt::Debug;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};
use std::{any::Any, cell::RefCell, sync::RwLock};

#[derive(Debug)]
pub struct ResourceCell2(Box<dyn SendableCell>);

impl ResourceCell2 {
    pub fn new<T: Resource + Send + Sync>(resource: T) -> Self {
        ResourceCell2(Box::new(RwLock::new(resource)))
    }

    pub fn read<T: Resource + Send + Sync>(&self) -> RwLockReadGuard<T> {
        self.0
            .to_any()
            .downcast_ref::<RwLock<T>>()
            .expect("donwcasting error")
            .try_read()
            .expect("lock error")
    }

    pub fn write<T: Resource + Send + Sync>(&self) -> RwLockWriteGuard<T> {
        self.0
            .to_any()
            .downcast_ref::<RwLock<T>>()
            .expect("donwcasting error")
            .try_write()
            .expect("lock error")
    }

    pub fn consume<T: Resource + Send + Sync>(self) -> Option<T> {
        let t = Box::into_raw(self.0);
        let t = unsafe { Box::from_raw(t.cast::<RwLock<T>>()) };

        Some(t.into_inner().expect("lock error"))
    }
}

trait SendableCell: Send + Sync + Debug {
    fn to_any(&self) -> &dyn Any;
}

impl<T: Resource + Send + Sync> SendableCell for RwLock<T> {
    fn to_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct UnsendableResourceCell2(Box<dyn UnsendableCell>);

unsafe impl Send for UnsendableResourceCell2 {}
unsafe impl Sync for UnsendableResourceCell2 {}

trait UnsendableCell: Debug {
    fn to_any(&self) -> &dyn Any;
}

impl<T: Resource> UnsendableCell for RefCell<T> {
    fn to_any(&self) -> &dyn Any {
        self
    }
}

impl UnsendableResourceCell2 {
    pub fn new<T: Resource>(resource: T) -> Self {
        UnsendableResourceCell2(Box::new(RefCell::new(resource)))
    }

    pub fn read<T: Resource>(&self) -> Ref<T> {
        self.0
            .to_any()
            .downcast_ref::<RefCell<T>>()
            .expect("donwcasting error")
            .try_borrow()
            .expect("lock error")
    }

    pub fn write<T: Resource>(&self) -> RefMut<T> {
        self.0
            .to_any()
            .downcast_ref::<RefCell<T>>()
            .expect("donwcasting error")
            .try_borrow_mut()
            .expect("lock error")
    }

    pub fn consume<T: Resource>(self) -> Option<T> {
        let t = Box::into_raw(self.0);
        let t = unsafe { Box::from_raw(t.cast::<RwLock<T>>()) };

        Some(t.into_inner().expect("lock error"))
    }
}

/// A data type that can be used to store resource in the [World](crate::World)
///
/// Resource is a [derivable trait](https://doc.rust-lang.org/book/appendix-03-derivable-traits.html):
/// you could implement it by applying a `#[derive(Resource)]` attribute to your data type.
/// To correctly implement this trait your data must satisfy the `Sync + Send + Debug` trait bounds.
///
/// # Implementing the trait for foreign types
/// As a consequence of the [orphan rule](https://doc.rust-lang.org/book/ch10-02-traits.html#implementing-a-trait-on-a-type),
/// it is not possible to separate into two different crates the implementation of Resource
/// from the definition of a type.
/// For this reason is not possible to implement the Resource trat for a type defined in a third party library.
/// The newtype pattern is a simple workaround to this limitation:
///
/// The following example gives a demonstration of this pattern.
/// ```
/// use zengine_macro::Resource;
///
/// #[derive(Resource, Debug)]
/// struct MyWrapper(Vec<usize>);
/// ```
pub trait Resource: Send + Sync + Any + Debug {}

#[doc(hidden)]
pub trait ResourceCell: Sync + Send + Debug {
    fn to_any(&self) -> &dyn Any;
    fn to_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Resource + Sync + Send> ResourceCell for RwLock<T> {
    fn to_any(&self) -> &dyn Any {
        self
    }
    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// A data type that can be used to store unsendable resource in the [World](crate::World)
///
/// UnsendableResource is a [derivable trait](https://doc.rust-lang.org/book/appendix-03-derivable-traits.html):
/// you could implement it by applying a `#[derive(UnsendableResource)]` attribute to your data type.
/// To correctly implement this trait your data must satisfy the `Debug` trait bounds.
///
/// # Implementing the trait for foreign types
/// As a consequence of the [orphan rule](https://doc.rust-lang.org/book/ch10-02-traits.html#implementing-a-trait-on-a-type),
/// it is not possible to separate into two different crates the implementation of UnsendableResource
/// from the definition of a type.
/// For this reason is not possible to implement the UnsendableResource trat for a type defined in a third party library.
/// The newtype pattern is a simple workaround to this limitation:
///
/// The following example gives a demonstration of this pattern.
/// ```
/// use zengine_macro::UnsendableResource;
///
/// #[derive(UnsendableResource, Debug)]
/// struct MyWrapper(Vec<usize>);
/// ```
pub trait UnsendableResource: Any + Debug {}

#[doc(hidden)]
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

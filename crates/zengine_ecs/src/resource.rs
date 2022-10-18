use std::fmt::Debug;
use std::{any::Any, cell::RefCell, sync::RwLock};

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
/// use external_crate::TypeThatShouldBeAResource;
///
/// #[derive(Resource)]
/// struct MyWrapper(TypeThatShouldBeAResource)
/// ```
pub trait Resource: Any + Sync + Send + Debug {}

#[doc(hidden)]
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
/// use external_crate::TypeThatShouldBeAUnsendableResource;
///
/// #[derive(UnsendableResource)]
/// struct MyWrapper(TypeThatShouldBeAUnsendableResource)
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

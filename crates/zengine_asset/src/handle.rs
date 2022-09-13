use std::{any::TypeId, marker::PhantomData};

pub type HandleId = u64;

pub struct Handle<T> {
    pub id: HandleId,
    pub type_id: TypeId,
    pub _phantom: PhantomData<T>,
}

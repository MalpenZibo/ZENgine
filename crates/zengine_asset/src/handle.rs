use std::{
    any::TypeId,
    cmp::Ordering,
    hash::{Hash, Hasher},
    marker::PhantomData,
};
use crossbeam_channel::{Receiver, Sender};
use log::debug;
use crate::{
    assets::{Asset, Assets},
    AssetPath,
};

/// A unique asset id
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd, Clone, Copy)]
pub enum HandleId {
    /// A handle id from a asset path
    FromPath(TypeId, u64),
    /// A handle id from an u64.
    FromU64(TypeId, u64),
}

impl HandleId {
    /// Creates a new HandleId from a given [AssetPath]
    pub fn new_from_path<T: Asset>(asset_path: &AssetPath) -> Self {
        let type_id = TypeId::of::<T>();

        let mut hasher = ahash::AHasher::default();
        asset_path.path.hash(&mut hasher);
        let id: u64 = hasher.finish();

        Self::FromPath(type_id, id)
    }

    /// Creates a new HandleId from a given u64
    pub fn new_from_u64<T: Asset>(id: u64) -> Self {
        let type_id = TypeId::of::<T>();

        Self::FromU64(type_id, id)
    }

    /// Clone and HandleId using a different [TypeId]
    pub fn clone_with_different_type<T: Asset>(&self) -> Self {
        let type_id = TypeId::of::<T>();

        match self {
            Self::FromPath(_, id) => Self::FromPath(type_id, *id),
            Self::FromU64(_, id) => Self::FromU64(type_id, *id),
        }
    }

    /// Get the [TypeId] of the HandleId
    pub fn get_type(&self) -> TypeId {
        match self {
            Self::FromPath(type_id, _) => *type_id,
            Self::FromU64(type_id, _) => *type_id,
        }
    }
}

impl<T: Asset> From<Handle<T>> for HandleId {
    fn from(value: Handle<T>) -> Self {
        value.id
    }
}

#[derive(PartialEq, Debug)]
pub(crate) enum HandleRef {
    Increment(HandleId),
    Decrement(HandleId),
}

#[derive(Default, Debug)]
pub(crate) enum HandleType {
    Strong(Sender<HandleRef>),
    #[default]
    Weak,
}

/// A handle into a specific [`Asset`] of type `T`
///
/// Handles contain a unique id that corresponds to a specific asset in the [`Assets`] collection.
///
/// # Accessing the Asset
///
/// A handle is _not_ the asset itself, it's mora like a pointer to the asset.
/// To get the actual asset, you should use [`Assets::get`] or [`Assets::get_mut`].
///
/// # Strong and Weak
///
/// A handle can be either "Strong" or "Weak".
/// A Strong handles keep the asset loaded.
///
/// A Weak handles do not affect the loaded status of assets.
///
/// This is due to a type of_reference counting_.
/// When the number of Strong handles that exist for any given asset reach
/// zero, the asset is dropped and will be unloaded.
///
/// If you want a reference to an asset but don't want to take the responsibility of
/// keeping it loaded that comes with a Strong handle then you need a Weak handle.
#[derive(Debug)]
pub struct Handle<T> {
    pub(crate) id: HandleId,
    handle_type: HandleType,
    _phantom: PhantomData<T>,
}

impl<T: Asset> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.id, state);
    }
}

impl<T: Asset> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: Asset> Eq for Handle<T> {}

impl<T: Asset> PartialOrd for Handle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl<T: Asset> Ord for Handle<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl<T> Drop for Handle<T> {
    fn drop(&mut self) {
        if let HandleType::Strong(sender) = &self.handle_type {
            let _res = sender.send(HandleRef::Decrement(self.id));
            debug!("Drop a strong handle id: {:?}", self.id);
        }
    }
}

impl<T: Asset> Clone for Handle<T> {
    fn clone(&self) -> Self {
        match self.handle_type {
            HandleType::Strong(ref sender) => Handle::strong(self.id, sender.clone()),
            HandleType::Weak => Handle::weak(self.id),
        }
    }
}

impl<T: Asset> Handle<T> {
    pub(crate) fn strong(id: HandleId, handle_ref_sender: Sender<HandleRef>) -> Self {
        handle_ref_sender.send(HandleRef::Increment(id)).unwrap();
        debug!("Create a strong handle id: {:?}", id);
        Self {
            id,
            handle_type: HandleType::Strong(handle_ref_sender),
            _phantom: PhantomData,
        }
    }

    /// Creates a weak handle for the Asset identified by `id`
    pub fn weak(id: HandleId) -> Self {
        Self {
            id,
            handle_type: HandleType::Weak,
            _phantom: PhantomData,
        }
    }

    /// Get the handle id
    pub fn get_id(&self) -> HandleId {
        self.id
    }

    /// Clone the handle producing a weak one that point to the same asset
    pub fn clone_as_weak(&self) -> Self {
        Handle::weak(self.id)
    }

    /// Convert an handle to a weak one
    pub fn as_weak(&self) -> Self {
        Handle::weak(self.id)
    }

    /// Checks if the handle is a strong one
    pub fn is_strong(&self) -> bool {
        matches!(self.handle_type, HandleType::Strong(_))
    }

    /// Checks if the handle is a weak one
    pub fn is_weak(&self) -> bool {
        matches!(self.handle_type, HandleType::Weak)
    }

    /// Makes this handle Strong if it wasn’t already
    ///
    /// This method requires the corresponding [Assets] storage
    pub fn make_strong(&mut self, assets: &Assets<T>) {
        if self.is_weak() {
            debug!("Create a strong handle from a weak one id: {:?}", self.id);
            let sender = assets.sender.clone();
            sender.send(HandleRef::Increment(self.id)).unwrap();
            self.handle_type = HandleType::Strong(sender)
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct HandleRefChannel {
    pub sender: Sender<HandleRef>,
    pub receiver: Receiver<HandleRef>,
}

impl Default for HandleRefChannel {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self { sender, receiver }
    }
}

#[cfg(test)]
mod tests {
    use crossbeam_channel::TryRecvError;

    use crate::{Asset, Assets, Handle, HandleId, HandleRef};

    #[derive(Debug)]
    pub struct TestAsset1 {}
    impl Asset for TestAsset1 {
        fn next_counter() -> u64
        where
            Self: Sized,
        {
            0
        }
    }

    #[derive(Debug)]
    pub struct TestAsset2 {}
    impl Asset for TestAsset2 {
        fn next_counter() -> u64
        where
            Self: Sized,
        {
            0
        }
    }

    #[test]
    fn handle_id_unique_constrain() {
        let id = HandleId::new_from_path::<TestAsset1>(&"path1.txt".into());
        let same_id = HandleId::new_from_path::<TestAsset1>(&"path1.txt".into());
        let different_id = HandleId::new_from_path::<TestAsset1>(&"path2.txt".into());

        assert_eq!(id, same_id);
        assert_ne!(id, different_id);

        let different_id = HandleId::new_from_path::<TestAsset2>(&"path1.txt".into());
        assert_ne!(id, different_id);
    }

    #[test]
    fn strong_handle_increment_ref_counter() {
        let (sender, receiver) = crossbeam_channel::unbounded::<HandleRef>();

        let id = HandleId::new_from_path::<TestAsset1>(&"path1.txt".into());
        let _handle: Handle<TestAsset1> = Handle::strong(id, sender);

        let handle_ref = receiver.try_recv();

        assert_eq!(handle_ref, Ok(HandleRef::Increment(id)));
    }

    #[test]
    fn strong_handle_is_a_strong_one() {
        let (sender, _receiver) = crossbeam_channel::unbounded::<HandleRef>();

        let id = HandleId::new_from_path::<TestAsset1>(&"path1.txt".into());
        let handle: Handle<TestAsset1> = Handle::strong(id, sender);

        assert!(handle.is_strong());
        assert!(!handle.is_weak());
    }

    #[test]
    fn weak_handle_is_a_weak_one() {
        let (sender, _receiver) = crossbeam_channel::unbounded::<HandleRef>();

        let id = HandleId::new_from_path::<TestAsset1>(&"path1.txt".into());
        let handle: Handle<TestAsset1> = Handle::strong(id, sender).as_weak();

        assert!(handle.is_weak());
        assert!(!handle.is_strong());
    }

    #[test]
    fn weak_handle_do_not_increment_ref_counter() {
        let (sender, receiver) = crossbeam_channel::unbounded::<HandleRef>();

        let id = HandleId::new_from_path::<TestAsset1>(&"path1.txt".into());
        let handle: Handle<TestAsset1> = Handle::strong(id, sender);
        let _handle2 = handle.as_weak();

        let handle_ref = receiver.try_recv();
        assert_eq!(handle_ref, Ok(HandleRef::Increment(id)));

        let handle_ref = receiver.try_recv();
        assert_eq!(handle_ref, Err(TryRecvError::Empty));
    }

    #[test]
    fn cloning_a_strong_handle_increment_ref_counter() {
        let (sender, receiver) = crossbeam_channel::unbounded::<HandleRef>();

        let id = HandleId::new_from_path::<TestAsset1>(&"path1.txt".into());
        let handle: Handle<TestAsset1> = Handle::strong(id, sender);

        let handle_ref = receiver.try_recv();
        assert_eq!(handle_ref, Ok(HandleRef::Increment(id)));

        #[allow(clippy::redundant_clone)]
        let _cloned_handle: Handle<TestAsset1> = handle.clone();

        let handle_ref = receiver.try_recv();
        assert_eq!(handle_ref, Ok(HandleRef::Increment(id)));
    }

    #[test]
    fn cloning_a_weak_handle_do_not_increment_ref_counter() {
        let (sender, receiver) = crossbeam_channel::unbounded::<HandleRef>();

        let id = HandleId::new_from_path::<TestAsset1>(&"path1.txt".into());
        let handle: Handle<TestAsset1> = Handle::strong(id, sender);

        let handle_ref = receiver.try_recv();
        assert_eq!(handle_ref, Ok(HandleRef::Increment(id)));

        #[allow(clippy::redundant_clone)]
        let _cloned_handle: Handle<TestAsset1> = handle.as_weak().clone();

        let handle_ref = receiver.try_recv();
        assert_eq!(handle_ref, Err(TryRecvError::Empty));
    }

    #[test]
    fn drop_a_strong_handle_decrement_ref_counter() {
        let (sender, receiver) = crossbeam_channel::unbounded::<HandleRef>();

        let id = HandleId::new_from_path::<TestAsset1>(&"path1.txt".into());
        {
            let _handle: Handle<TestAsset1> = Handle::strong(id, sender);

            let handle_ref = receiver.try_recv();
            assert_eq!(handle_ref, Ok(HandleRef::Increment(id)));
        }

        let handle_ref = receiver.try_recv();
        assert_eq!(handle_ref, Ok(HandleRef::Decrement(id)));
    }

    #[test]
    fn drop_a_weak_handle_do_not_decrement_ref_counter() {
        let (sender, receiver) = crossbeam_channel::unbounded::<HandleRef>();

        let id = HandleId::new_from_path::<TestAsset1>(&"path1.txt".into());
        let handle: Handle<TestAsset1> = Handle::strong(id, sender);

        let handle_ref = receiver.try_recv();
        assert_eq!(handle_ref, Ok(HandleRef::Increment(id)));

        {
            let _weak_handle = handle.as_weak();
        }

        let handle_ref = receiver.try_recv();
        assert_eq!(handle_ref, Err(TryRecvError::Empty));
    }

    #[test]
    fn making_a_weak_handle_a_strong_one_increment_ref_counter() {
        let (sender, receiver) = crossbeam_channel::unbounded::<HandleRef>();

        let assets: Assets<TestAsset1> = Assets::new(sender);

        let id = HandleId::new_from_path::<TestAsset1>(&"path1.txt".into());
        let mut handle: Handle<TestAsset1> = Handle::weak(id);

        handle.make_strong(&assets);

        assert!(handle.is_strong());

        let handle_ref = receiver.try_recv();
        assert_eq!(handle_ref, Ok(HandleRef::Increment(id)));
    }

    #[test]
    fn making_a_strong_handle_a_strong_one_do_not_increment_ref_counter() {
        let (sender, receiver) = crossbeam_channel::unbounded::<HandleRef>();

        let assets: Assets<TestAsset1> = Assets::new(sender.clone());

        let id = HandleId::new_from_path::<TestAsset1>(&"path1.txt".into());
        let mut handle: Handle<TestAsset1> = Handle::strong(id, sender);

        let handle_ref = receiver.try_recv();
        assert_eq!(handle_ref, Ok(HandleRef::Increment(id)));

        handle.make_strong(&assets);

        assert!(handle.is_strong());

        let handle_ref = receiver.try_recv();
        assert_eq!(handle_ref, Err(TryRecvError::Empty));
    }
}

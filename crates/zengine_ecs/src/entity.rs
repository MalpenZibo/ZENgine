use std::{
    ops::Deref,
    sync::atomic::{AtomicUsize, Ordering},
};

/// Identifier of an entity
///
/// An entity owns zero or more [Component](crate::Component) instances,
/// all of different types, and can dynamically acquire or lose them over its lifetime.
///
/// # Usage
/// This data type is returned by iterating a [Query](crate::query::Query) that has
/// Entity as part of its query fetch type parameter.
///
/// ```
/// use zengine_macro::Component;
/// use zengine_ecs::{Entity, query::{Query, QueryIter}};
///
/// #[derive(Component, Debug)]
/// struct TestComponent {}
///
/// fn example_system(query: Query<(Entity, &TestComponent)>) {
///     for (entity, test_component) in query.iter() {
///         println!("test_component: {:?} for entity: {:?}", test_component, entity);
///     }
/// }
/// ```
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Entity(usize);
impl Deref for Entity {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default, Debug)]
pub(crate) struct EntityGenerator {
    current: AtomicUsize,
}
impl EntityGenerator {
    pub fn generate(&self) -> Entity {
        let current = self.current.fetch_add(1, Ordering::Relaxed);
        Entity(current)
    }
}

#[cfg(test)]
mod tests {
    use super::EntityGenerator;

    #[test]
    fn generate_an_entity() {
        let mut generator = EntityGenerator::default();

        let entities = vec![
            generator.generate(),
            generator.generate(),
            generator.generate(),
            generator.generate(),
        ];

        assert_eq!(
            entities.into_iter().map(|e| e.0).collect::<Vec<usize>>(),
            vec!(0, 1, 2, 3)
        );
        assert_eq!(*generator.current.get_mut(), 4);
    }
}
